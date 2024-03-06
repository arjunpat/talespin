use anyhow::{anyhow, Result};
use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket},
        Json, State, WebSocketUpgrade,
    },
    http::{header, Method},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::{collections::HashMap, fs, net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

mod room;

use rand::distributions::{Distribution, Uniform};
use room::{get_time_s, Room, ServerMsg};

const GARBAGE_COLLECT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 20); // 20 minutes
const GC_ROOM_TIMEOUT_S: u64 = 60 * 60; // 1 hour

// main object for server
#[derive(Debug, Clone)]
struct ServerState {
    rooms: DashMap<String, Arc<Room>>,
    base_deck: Arc<Vec<String>>,
}

impl ServerState {
    fn new() -> Result<Self> {
        // read cards and form array of file names, any file is ok
        let base_deck: Vec<String> = fs::read_dir("../static/assets/cards/")?
            .map(|res| res.map(|e| e.file_name().into_string().unwrap()))
            .map(|res| res.unwrap())
            .filter(|s| s.ends_with(".jpg") || s.ends_with(".jpeg") || s.ends_with(".png"))
            .collect();

        println!("Loaded {} cards", base_deck.len());

        Ok(ServerState {
            rooms: DashMap::new(),
            base_deck: Arc::new(base_deck),
        })
    }

    async fn create_room(&self) -> Result<ServerMsg> {
        let mut room_id = generate_room_id(4);

        // println!("create room: 0");
        while (self.get_room(&room_id)).is_some() {
            room_id = generate_room_id(4);
        }

        let room = Room::new(&room_id, self.base_deck.clone());
        let msg = room.get_room_state().await;
        self.rooms.insert(room_id.clone(), Arc::new(room));
        Ok(msg)
    }

    async fn join_room(&self, room_id: &str, socket: &mut WebSocket, name: &str) -> Result<()> {
        // hold no reference to inside the dashmap to prevent deadlock
        if let Some(room) = self.get_room(room_id) {
            room.on_connection(socket, name).await;
        } else {
            socket.send(ServerMsg::InvalidRoomId {}.into()).await?;
            return Ok(());
        }

        Ok(())
    }

    fn get_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.get(room_id).map(|r| r.value().clone())
    }

    fn stats(&self) -> HashMap<String, (usize, u64)> {
        self.rooms
            .iter()
            .map(|r| {
                (
                    r.key().clone(),
                    (r.value().num_active(), r.value().last_access()),
                )
            })
            .collect()
    }

    fn garbage_collect(&self) {
        let mut to_remove = Vec::new();
        for entry in &self.rooms {
            // hasn't been accessed in an hour
            if entry.value().num_active() == 0
                && get_time_s() - entry.value().last_access() > GC_ROOM_TIMEOUT_S
            {
                to_remove.push(entry.key().clone());
            }
        }

        println!("(gc) rooms to delete {:?}", to_remove);
        for room_id in to_remove {
            self.rooms.remove(&room_id);
        }
    }
}

async fn garbage_collect(state: Arc<ServerState>) {
    loop {
        tokio::time::sleep(GARBAGE_COLLECT_INTERVAL).await;
        state.garbage_collect();
    }
}

fn generate_room_id(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let letters = Uniform::new_inclusive(b'a', b'z'); // Range of lowercase letters
    (0..length)
        .map(|_| letters.sample(&mut rng) as char)
        .collect()
}

#[tokio::main]
async fn main() {
    let state = Arc::new(ServerState::new().unwrap());

    tokio::spawn(garbage_collect(state.clone()));

    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/create", post(create_room_handler))
        .route("/exists", post(exists_handler))
        .route("/stats", get(stats_handler))
        .route("/", get(root))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn create_room_handler(State(state): State<Arc<ServerState>>) -> String {
    let room = state.create_room().await;
    // json response with room id

    if let Ok(room_state) = room {
        serde_json::to_string(&room_state).unwrap()
    } else {
        serde_json::to_string(&room::ServerMsg::ErrorMsg(
            "Failed to create room".to_string(),
        ))
        .unwrap()
    }
}

async fn exists_handler(
    State(state): State<Arc<ServerState>>,
    Json(room_id): Json<String>,
) -> &'static str {
    if state.get_room(&room_id).is_some() {
        "true"
    } else {
        "false"
    }
}

async fn stats_handler(State(state): State<Arc<ServerState>>) -> String {
    serde_json::to_string(&state.stats()).unwrap()
}

async fn root() -> &'static str {
    "Hello, world!"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<ServerState>) {
    let res = initialize_socket(&mut socket, state).await;

    if let Err(e) = res {
        println!("Error in initialize_socket: {}", e);
    }
}

async fn initialize_socket(socket: &mut WebSocket, state: Arc<ServerState>) -> Result<()> {
    let msg = socket
        .recv()
        .await
        .ok_or_else(|| anyhow!("Expected initial message from client"))??;

    if let WsMessage::Text(s) = msg {
        if let Ok(msg) = serde_json::from_str(&s) {
            if let room::ClientMsg::JoinRoom { room_id, name } = msg {
                if name.len() > 30 {
                    socket
                        .send(room::ServerMsg::ErrorMsg("Name too long".to_string()).into())
                        .await?;
                    return Err(anyhow!("Name too long"));
                }
                state
                    .join_room(&room_id.to_lowercase(), socket, &name)
                    .await?
            }
        }
    }

    Ok(())
}
