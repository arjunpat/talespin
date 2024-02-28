use anyhow::{anyhow, Context, Result};
use axum::{
    extract::ws::Message as WsMessage,
    extract::State,
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

use rand::distributions::{Distribution, Uniform};

// main object for server
#[derive(Debug, Clone)]
struct ServerState {
    rooms: Arc<DashMap<String, Arc<Room>>>,
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            rooms: Arc::new(DashMap::new()),
        }
    }

    async fn create_room(&self) -> Result<String> {
        let room_id = generate_room_id(4);
        let room = Room::new(&room_id);
        self.rooms.insert(room_id.clone(), Arc::new(room));
        Ok(room_id)
    }

    async fn join_room(&self, room_id: &str, socket: &mut WebSocket, name: &str) -> Result<()> {
        let room = self
            .rooms
            .get(room_id)
            .ok_or_else(|| anyhow!("Room not found"))?;
        room.handle_join(socket, name).await;
        Ok(())
    }
}

fn generate_room_id(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let letters = Uniform::new_inclusive(b'a', b'z'); // Range of lowercase letters
    (0..length)
        .map(|_| letters.sample(&mut rng) as char)
        .collect()
}

#[derive(Debug, Clone, Copy)]
enum RoomStage {
    Joining,
}
#[derive(Debug)]
struct RoomState {
    points: HashMap<String, u32>,
    room_id: String,
    players: Vec<String>,
    stage: RoomStage,
}

// main object representing a game
#[derive(Debug)]
struct Room {
    state: RwLock<RoomState>,
}

impl Room {
    fn new(room_id: &str) -> Self {
        let state = RoomState {
            points: HashMap::new(),
            room_id: room_id.to_string(),
            players: Vec::new(),
            stage: RoomStage::Joining,
        };
        Room {
            state: RwLock::new(state),
        }
    }

    async fn handle_join(&self, socket: &mut WebSocket, name: &str) {
        let stage;
        {
            stage = self.state.read().await.stage;
        }

        let res = match stage {
            RoomStage::Joining => self.handle_first_join(socket, name).await,
            _ => self.handle_rejoin(socket, name).await,
        };
    }

    async fn handle_rejoin(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
        println!("Handling rejoin for {}", name);
        Ok(())
    }

    async fn handle_first_join(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
        println!("Handling join for {}", name);
        {
            let mut state = self.state.write().await;

            if state.players.contains(&name.to_string()) {
                socket
                    .send(ServerMsg::Error("Name already taken".to_string()).into())
                    .await?;
                return Err(anyhow!("Name already taken"));
            }

            state.players.push(name.to_string());
            state.points.insert(name.to_string(), 0);

            // send initial room state to client
            let msg = ServerMsg::RoomState {
                room_id: state.room_id.clone(),
                points: state.points.clone(),
                players: state.players.clone(),
            };
            socket.send(msg.into()).await?;
        }

        loop {
            tokio::select! {
            msg = socket.recv() => {
                    if let Some(msg) = msg {
                        if let Ok(msg) = msg {
                            if let WsMessage::Text(text) = msg {
                                self.handle_client_msg(&name, text).await?
                            }
                        } else {
                            println!("1. Client disconnected!!!!");
                            break;
                        }
                    } else {
                        println!("2. Stream has closed");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_client_msg(&self, name: &str, msg: String) -> Result<()> {
        let msg: ClientMsg = serde_json::from_str(&msg)
            .context(format!("Failed to deserialize client msg: {}", msg))?;

        println!("Handling client message: {:?}", msg);

        Ok(())
    }
}

#[derive(Debug, Serialize)]
enum ServerMsg {
    RoomState {
        room_id: String,
        points: HashMap<String, u32>,
        players: Vec<String>,
    },
    Error(String),
}

impl From<ServerMsg> for WsMessage {
    fn from(msg: ServerMsg) -> Self {
        // this should never fail
        let json = serde_json::to_string(&msg).expect("Failed to serialize json");
        WsMessage::Text(json)
    }
}

#[derive(Debug, Deserialize)]
enum ClientMsg {
    JoinRoom { room_id: String, name: String },
    CreateRoom { name: String },
}

#[tokio::main]
async fn main() {
    let state = ServerState::new();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/", get(test_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<ServerState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: ServerState) {
    // let _ = socket.send(WsMessage::Ping(vec![])).await;

    let res = initialize_socket(&mut socket, state).await;

    if let Err(e) = res {
        println!("Error: {}", e);
    }
}

async fn initialize_socket(socket: &mut WebSocket, state: ServerState) -> Result<()> {
    let msg = socket
        .recv()
        .await
        .ok_or_else(|| anyhow!("Expected initial message from client"))??;

    match msg {
        WsMessage::Text(s) => {
            let msg: ClientMsg = serde_json::from_str(&s)
                .context(format!("Failed to deserialize client msg: {}", s))?;

            match msg {
                ClientMsg::JoinRoom { room_id, name } => {
                    println!("Joining room: {} as {}", room_id, name);
                    // TODO double check this code because the error might not always be failed to join room
                    if let Err(e) = state.join_room(&room_id, socket, &name).await {
                        println!("Error joining room: {}", e);
                        socket
                            .send(ServerMsg::Error("Failed to join room".to_string()).into())
                            .await?;
                    }
                }
                ClientMsg::CreateRoom { name } => {
                    println!("Creating room as {}", name);
                    let room_id = state.create_room().await?;
                    if let Err(e) = state.join_room(&room_id, socket, &name).await {
                        println!("Error after creating room: {}", e);
                    }
                }
                _ => {
                    return Err(anyhow!(
                        "Expected JoinRoom or CreateRoom message from client"
                    ));
                }
            }
        }
        _ => {
            return Err(anyhow!("Expected text message from client"));
        }
    };

    Ok(())
}

async fn test_handler() -> &'static str {
    "Hello, World!"
}
