use anyhow::{anyhow, Context, Result};
use axum::{extract::ws::Message as WsMessage, extract::ws::WebSocket};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc, RwLock, RwLockWriteGuard};

#[derive(Debug, Serialize, Clone)]
pub enum ServerMsg {
    RoomState {
        room_id: String,
        players: HashMap<String, PlayerInfo>,
        stage: RoomStage,
    },
    StartRound {
        active_player: String,
        hand: Vec<String>,
    },
    PlayersChoose {
        active_player: String,
        description: String,
        deadline: u64,
    },
    BeginVoting {
        center_cards: Vec<String>,
        deadline: u64,
    },
    Results {
        player_to_vote: HashMap<String, String>,
        active_card: String,
        point_change: HashMap<String, u16>,
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
    StartRound,
    ActivePlayerChooseCard { card: String, description: String },
    PlayerChooseCard { card: String },
    Vote { card: String },
}

#[derive(Debug, Serialize, Clone, Copy)]
pub enum RoomStage {
    Joining,
    ActiveChooses,
    PlayersChoose,
    Voting,
    Results,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerInfo {
    active: bool,
    points: u16,
}

#[derive(Debug)]
struct RoomState {
    room_id: String,
    players: HashMap<String, PlayerInfo>,
    player_hand: HashMap<String, Vec<String>>,
    deck: Vec<String>,
    stage: RoomStage,
    player_order: Vec<String>,
    player_to_socket: HashMap<String, mpsc::Sender<ServerMsg>>,
    active_player: usize,
    current_description: String,
    player_to_current_card: HashMap<String, String>,
    player_to_vote: HashMap<String, String>,
    answer_deadline: u64,
}

// main object representing a game
#[derive(Debug)]
pub struct Room {
    // store all informationa about the room
    state: RwLock<RoomState>,
    // send updates to everyone in the room
    broadcast: broadcast::Sender<ServerMsg>,
}

impl Room {
    pub fn new(room_id: &str, deck: Vec<String>) -> Self {
        let state = RoomState {
            room_id: room_id.to_string(),
            players: HashMap::new(),
            deck,
            stage: RoomStage::Joining,
            player_order: Vec::new(),
            player_hand: HashMap::new(),
            player_to_socket: HashMap::new(),
            active_player: 0,
            current_description: "".to_string(),
            player_to_current_card: HashMap::new(),
            player_to_vote: HashMap::new(),
            answer_deadline: u64::MAX,
        };

        let (tx, _) = broadcast::channel(10);

        Self {
            state: RwLock::new(state),
            broadcast: tx,
        }
    }

    pub async fn handle_client_msg(&self, name: &str, msg: WsMessage) -> Result<()> {
        let mut state = self.state.write().await;
        {
            // check if answer deadline has passed
            if state.answer_deadline < tokio::time::Instant::now().elapsed().as_secs() {
                if matches!(state.stage, RoomStage::PlayersChoose) {
                    state.stage = RoomStage::Voting;

                    // remove cards from hand that were put in the center
                    for (player, card) in state.player_to_current_card.clone().iter() {
                        if let Some(hand) = state.player_hand.get_mut(player) {
                            if let Some(pos) = hand.iter().position(|e| e == card) {
                                hand.remove(pos);
                            }
                        }
                    }

                    state.answer_deadline = 45 + tokio::time::Instant::now().elapsed().as_secs();
                    let mut center_cards: Vec<String> = state
                        .player_to_current_card
                        .values()
                        .map(|e| e.to_string())
                        .collect();
                    center_cards.shuffle(&mut rand::thread_rng());

                    self.broadcast.send(ServerMsg::BeginVoting {
                        center_cards,
                        deadline: state.answer_deadline,
                    })?;
                } else if matches!(state.stage, RoomStage::Voting) {
                    state.stage = RoomStage::Results;
                    self.send_results(&mut state);
                }
            }
        }

        let msg: ClientMsg = serde_json::from_str(msg.to_text()?)
            .context(format!("Failed to deserialize client msg: {:?}", msg))?;

        println!("Handling client message: {:?}", msg);

        match msg {
            ClientMsg::StartRound => {
                if matches!(state.stage, RoomStage::Joining) {
                    // finalize players
                    if state.player_order.len() == 0 {
                        // first round
                        state.active_player = 0;
                        state.player_order = state.players.keys().cloned().collect::<Vec<_>>();
                        state.player_order.shuffle(&mut rand::thread_rng());
                    } else {
                        state.active_player = (state.active_player + 1) % state.player_order.len();
                    }

                    // shuffle deck
                    state.deck.shuffle(&mut rand::thread_rng());

                    // clear current chosen cards
                    state.player_to_current_card.clear();
                    state.player_to_vote.clear();

                    // ensure all players have 6 cards
                    let mut player_hand = state.player_hand.clone();

                    let mut deck = state.deck.clone();
                    for player in state.players.keys() {
                        if !player_hand.contains_key(player) {
                            player_hand.insert(player.clone(), Vec::new());
                        }

                        while player_hand.get(player).unwrap().len() < 6 {
                            player_hand.get_mut(player).unwrap().push(
                                deck.pop()
                                    .ok_or_else(|| anyhow!("Not enough cards in the deck"))?,
                            );
                        }
                    }

                    // notify players of the game start and their hands
                    for (player, hand) in player_hand.iter() {
                        if let Some(tx) = state.player_to_socket.get(player) {
                            tx.send(ServerMsg::StartRound {
                                hand: hand.clone(),
                                active_player: state.player_order[state.active_player].clone(),
                            })
                            .await?;
                        }
                    }

                    state.deck = deck;
                    state.player_hand = player_hand;

                    state.stage = RoomStage::ActiveChooses;
                    self.broadcast.send(self.server_msg(&state))?;
                }
            }
            ClientMsg::ActivePlayerChooseCard { card, description } => {
                if matches!(state.stage, RoomStage::ActiveChooses) {
                    if state.player_order[state.active_player] == name {
                        // verify that player has this card
                        if !state.player_hand.get(name).unwrap().contains(&card) {
                            return Err(anyhow!("Invalid card chosen by active player"));
                        }

                        let description = description.trim();
                        // verify that the description is not empty and is one word
                        if description.is_empty() || description.contains(' ') {
                            return Err(anyhow!("Invalid description chosen by active player"));
                        }
                        state.current_description = description.to_string();

                        state.answer_deadline =
                            45 + tokio::time::Instant::now().elapsed().as_secs();

                        // notify players of the active player's choice
                        self.broadcast.send(ServerMsg::PlayersChoose {
                            active_player: name.to_string(),
                            description: description.to_string(),
                            deadline: state.answer_deadline,
                        })?;

                        state.stage = RoomStage::PlayersChoose;
                        self.broadcast.send(self.server_msg(&state))?;
                    }
                }
            }
            ClientMsg::PlayerChooseCard { card } => {
                if matches!(state.stage, RoomStage::PlayersChoose) {
                    if state.player_order[state.active_player] != name {
                        // verify that player has this card
                        if !state.player_hand.get(name).unwrap().contains(&card) {
                            return Err(anyhow!("Invalid card chosen by player"));
                        }

                        // record choice
                        state
                            .player_to_current_card
                            .insert(name.to_string(), card.to_string());
                    }
                }
            }
            ClientMsg::Vote { card } => {
                if matches!(state.stage, RoomStage::Voting) {
                    // verify that the player is not the active player
                    if state.player_order[state.active_player] != name {
                        return Err(anyhow!("Active player cannot vote"));
                    }

                    // verify that the card is in the center
                    if !state.player_to_current_card.values().any(|e| e == &card) {
                        return Err(anyhow!("Invalid card voted for"));
                    }

                    // record vote
                    state
                        .player_to_vote
                        .insert(name.to_string(), card.to_string());
                }
            }
            _ => {
                // nothing
            }
        }

        Ok(())
    }

    fn send_results(&self, state: &mut RwLockWriteGuard<RoomState>) {
        let mut point_change: HashMap<String, u16> = HashMap::new();
        let active_player = state.player_order[state.active_player].clone();
        let active_card = state
            .player_to_current_card
            .get(&active_player)
            .unwrap()
            .clone();

        let mut votes_for_card: HashMap<String, u16> = HashMap::new();

        for (player, card) in state.player_to_vote.iter() {
            *votes_for_card.entry(card.to_string()).or_insert(0) += 1;
        }

        let votes_for_active_card = *votes_for_card.get(&active_card).unwrap_or(&0);
        if votes_for_active_card == 0 || votes_for_active_card == state.player_to_vote.len() as u16
        {
            // everyone or no-one voted
            for (player, _) in state.player_to_vote.iter() {
                point_change.insert(player.to_string(), 2);
            }
            point_change.insert(active_player, 0);
        } else {
            for (player, card) in state.player_to_current_card.iter() {
                if card == &active_card {
                    point_change.insert(player.to_string(), 3);
                } else {
                    point_change.insert(player.to_string(), 0);
                }

                *point_change.get_mut(player).unwrap() += votes_for_card.get(card).unwrap_or(&0);
            }
        }

        for (player, points) in point_change.iter() {
            if let Some(info) = state.players.get_mut(player) {
                info.points += points;
            }
        }

        let res = self.broadcast.send(ServerMsg::Results {
            player_to_vote: state.player_to_vote.clone(),
            active_card: active_card.to_string(),
            point_change,
        });
        self.broadcast.send(self.server_msg(&state)).unwrap();
    }

    pub async fn on_connection(&self, socket: &mut WebSocket, name: &str) {
        // public funciton

        let res = self.handle_join(socket, name).await;
        println!("Player {} has left", name);

        let mut state = self.state.write().await;

        if matches!(state.stage, RoomStage::Joining) {
            state.players.remove(name);
        } else {
            if let Some(player) = state.players.get_mut(name) {
                player.active = false;
            }
        }

        state.player_to_socket.remove(name);

        if let Err(e) = res {
            println!("{}", e);
        }

        if let Err(e) = self.broadcast.send(self.server_msg(&state)) {
            println!("Error sending broadcast: {}", e);
        }
    }

    async fn handle_join(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
        println!("Handling join for {}", name);

        let (tx, mut rx) = mpsc::channel(10);

        {
            let mut state = self.state.write().await;

            if let Some(player) = state.players.get_mut(name) {
                // player already exists in the game
                // and not in joining anymore
                // if in joining then player.active will be true

                if !player.active {
                    player.active = true;
                } else {
                    socket
                        .send(ServerMsg::Error("Name already taken".to_string()).into())
                        .await?;
                    return Err(anyhow!("Name already taken"));
                }
            } else if matches!(state.stage, RoomStage::Joining) {
                // still in joining and not yet joined
                if state.players.len() < 8 {
                    state.players.insert(
                        name.to_string(),
                        PlayerInfo {
                            active: true,
                            points: 0,
                        },
                    );
                } else {
                    socket
                        .send(ServerMsg::Error("Too many players!".to_string()).into())
                        .await?;
                    return Err(anyhow!("Too many players!"));
                }
            } else {
                socket
                    .send(ServerMsg::Error("Game has already started".to_string()).into())
                    .await?;
                return Err(anyhow!("Game has already started"));
            }

            self.broadcast.send(self.server_msg(&state))?;
            state.player_to_socket.insert(name.to_string(), tx);
        }

        let mut broadcast_updates = self.broadcast.subscribe();

        loop {
            tokio::select! {
                msg = broadcast_updates.recv() => {
                    socket.send(msg?.into()).await?;
                }
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(msg)) => {
                            self.handle_client_msg(name, msg).await?;
                        }
                        _ => break
                    }
                },
                msg = rx.recv() => {
                    if let Some(msg) = msg {
                        socket.send(msg.into()).await?;
                    } else {
                        // channel has been closed
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn server_msg(&self, state: &RwLockWriteGuard<RoomState>) -> ServerMsg {
        ServerMsg::RoomState {
            room_id: state.room_id.clone(),
            players: state.players.clone(),
            stage: state.stage,
        }
    }
}
