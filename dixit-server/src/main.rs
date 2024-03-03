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
use std::{fs, net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

mod room;

use rand::distributions::{Distribution, Uniform};
use room::{Room, ServerMsg};

// main object for server
#[derive(Debug, Clone)]
struct ServerState {
    rooms: Arc<DashMap<String, Arc<Room>>>,
    cards: Vec<String>,
}

impl ServerState {
    fn new() -> Result<Self> {
        // read cards and form array of file names, any file is ok
        let cards: Vec<String> = fs::read_dir("../static/assets/cards/")?
            .map(|res| res.map(|e| e.file_name().into_string().unwrap()))
            .map(|res| res.unwrap())
            .filter(|s| s.ends_with(".jpg") || s.ends_with(".jpeg") || s.ends_with(".png"))
            .collect();

        println!("Loaded {} cards", cards.len());

        Ok(ServerState {
            rooms: Arc::new(DashMap::new()),
            cards,
        })
    }

    async fn create_room(&self) -> Result<ServerMsg> {
        let room_id = generate_room_id(4);

        let room = Room::new(&room_id, self.cards.clone());
        let msg = room.get_room_state().await;
        self.rooms.insert(room_id.clone(), Arc::new(room));
        Ok(msg)
    }

    async fn join_room(&self, room_id: &str, socket: &mut WebSocket, name: &str) -> Result<()> {
        if let Some(room) = self.rooms.get(room_id) {
            room.value().on_connection(socket, name).await;
        } else {
            socket
                .send(room::ServerMsg::InvalidRoomId {}.into())
                .await?;
        }
        Ok(())
    }

    fn get_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.get(room_id).map(|r| r.value().clone())
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
    let state = ServerState::new().unwrap();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/create", post(create_room))
        .route("/exists", post(exists_handler))
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

async fn create_room(State(state): State<ServerState>) -> String {
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
    State(state): State<ServerState>,
    Json(room_id): Json<String>,
) -> &'static str {
    if state.get_room(&room_id).is_some() {
        "true"
    } else {
        "false"
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<ServerState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: ServerState) {
    let res = initialize_socket(&mut socket, state).await;

    if let Err(e) = res {
        println!("Error in initialize_socket: {}", e);
    }
}

async fn initialize_socket(socket: &mut WebSocket, state: ServerState) -> Result<()> {
    let msg = socket
        .recv()
        .await
        .ok_or_else(|| anyhow!("Expected initial message from client"))??;

    if let WsMessage::Text(s) = msg {
        if let Ok(msg) = serde_json::from_str(&s) {
            if let room::ClientMsg::JoinRoom { room_id, name } = msg {
                println!("Attmping: join {} as {}", room_id, name);
                state.join_room(&room_id, socket, &name).await?
            }
        }
    }

    Ok(())
}
