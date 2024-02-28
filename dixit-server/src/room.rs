use anyhow::{anyhow, Context, Result};
use axum::{extract::ws::Message as WsMessage, extract::ws::WebSocket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Serialize)]
pub enum ServerMsg {
    RoomState {
        room_id: String,
        points: HashMap<String, u32>,
        players: Vec<String>,
        stage: RoomStage,
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
pub enum ClientMsg {
    JoinRoom { room_id: String, name: String },
    CreateRoom { name: String },
}

#[derive(Debug, Serialize, Clone, Copy)]
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
pub struct Room {
    state: RwLock<RoomState>,
}

impl Room {
    pub fn new(room_id: &str) -> Self {
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

    pub async fn handle_join(&self, socket: &mut WebSocket, name: &str) {
        let stage;
        {
            stage = self.state.read().await.stage;
        }

        let res = match stage {
            RoomStage::Joining => self.handle_first_join(socket, name).await,
            _ => self.handle_rejoin(socket, name).await,
        };
    }

    pub async fn handle_rejoin(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
        println!("Handling rejoin for {}", name);
        Ok(())
    }

    pub async fn handle_first_join(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
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
                stage: state.stage,
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

    pub async fn handle_client_msg(&self, name: &str, msg: String) -> Result<()> {
        let msg: ClientMsg = serde_json::from_str(&msg)
            .context(format!("Failed to deserialize client msg: {}", msg))?;

        println!("Handling client message: {:?}", msg);

        Ok(())
    }
}
