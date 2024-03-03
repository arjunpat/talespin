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
        active_player: Option<String>,
        player_order: Vec<String>,
    },
    StartRound {
        hand: Vec<String>,
    },
    PlayersChoose {
        description: String,
        hand: Vec<String>,
    },
    BeginVoting {
        center_cards: Vec<String>,
        description: String,
    },
    Results {
        player_to_vote: HashMap<String, String>,
        player_to_current_card: HashMap<String, String>,
        active_card: String,
        point_change: HashMap<String, u16>,
    },
    ErrorMsg(String),
    InvalidRoomId {},
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
    Ready {},
    JoinRoom { room_id: String, name: String },
    CreateRoom { name: String },
    ActivePlayerChooseCard { card: String, description: String },
    PlayerChooseCard { card: String },
    Vote { card: String },
    Ping {},
}

#[derive(Debug, Serialize, Clone, Copy)]
pub enum RoomStage {
    // waiting for players to join with room code
    Joining,
    // active player chooses card from their hand and writes description
    ActiveChooses,
    // players choose cards to match the description
    PlayersChoose,
    // players vote on what they think the active card is
    Voting,
    // results are computed; circle back to ActiveChooses while deck is not empty
    Results,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerInfo {
    // player is connected to server
    connected: bool,
    // points in the game
    points: u16,
    // ready is stage-specific
    ready: bool, // this is round dependent
}

#[derive(Debug)]
struct RoomState {
    room_id: String,
    // store general stats about each player
    players: HashMap<String, PlayerInfo>,
    // store 6 cards in hand per player
    player_hand: HashMap<String, Vec<String>>,
    // remaining deck; pop from this to players hands
    deck: Vec<String>,
    // stage of the game
    stage: RoomStage,
    // order of players being "active"
    player_order: Vec<String>,
    active_player: usize, // index into player_order
    // map to mpsc which sends messages to specific players
    player_to_socket: HashMap<String, mpsc::Sender<ServerMsg>>,

    /** Round-specific information */
    // chosen description by active player
    current_description: String,
    // for the active player, this is the active card; for other players, this is the card they chose
    player_to_current_card: HashMap<String, String>,
    // for each player, the card they voted for as being the active's card
    // they cannot vote for themselves
    player_to_vote: HashMap<String, String>,
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
        };

        let (tx, _) = broadcast::channel(10);

        Self {
            state: RwLock::new(state),
            broadcast: tx,
        }
    }

    fn get_msg(
        &self,
        name: Option<&str>,
        state: &RwLockWriteGuard<RoomState>,
    ) -> Result<ServerMsg> {
        match state.stage {
            RoomStage::ActiveChooses => Ok(ServerMsg::StartRound {
                hand: state.player_hand[name.ok_or_else(|| anyhow!("No name provided"))?].clone(),
            }),
            RoomStage::PlayersChoose => Ok(ServerMsg::PlayersChoose {
                description: state.current_description.clone(),
                hand: state.player_hand[name.ok_or_else(|| anyhow!("No name provided"))?].clone(),
            }),
            RoomStage::Voting => Ok(ServerMsg::BeginVoting {
                center_cards: self.get_center_cards(state),
                description: state.current_description.clone(),
            }),
            RoomStage::Results => Ok(ServerMsg::Results {
                player_to_vote: state.player_to_vote.clone(),
                player_to_current_card: state.player_to_current_card.clone(),
                active_card: state
                    .player_to_current_card
                    .get(&self.get_active_player(state)?)
                    .unwrap()
                    .to_string(),
                point_change: self.compute_results(state),
            }),
            _ => Err(anyhow!("No msg to send")),
        }
    }

    fn get_center_cards(&self, state: &RwLockWriteGuard<RoomState>) -> Vec<String> {
        let mut center_cards: Vec<String> = state
            .player_to_current_card
            .values()
            .map(|e| e.to_string())
            .collect();
        center_cards.shuffle(&mut rand::thread_rng());
        center_cards
    }

    fn get_active_player(&self, state: &RwLockWriteGuard<RoomState>) -> Result<String> {
        if matches!(state.stage, RoomStage::Joining) {
            return Err(anyhow!("Failed to find active player"));
        }

        Ok(state
            .player_order
            .get(state.active_player)
            .unwrap() // unreachable
            .to_string())
    }

    fn init_voting(&self, state: &mut RwLockWriteGuard<RoomState>) -> Result<()> {
        state.stage = RoomStage::Voting;

        // choose random card for those who didn't choose by the deadline
        for player in state.player_order.clone().iter() {
            if !state.player_to_current_card.contains_key(player) {
                let mut rng = rand::thread_rng();
                let card = state.player_hand[player].choose(&mut rng).unwrap().clone();
                state
                    .player_to_current_card
                    .insert(player.to_string(), card);
            }
        }

        self.clear_ready(state);

        // remove cards from hand that were put in the center
        for (player, card) in state.player_to_current_card.clone().iter() {
            if let Some(hand) = state.player_hand.get_mut(player) {
                if let Some(pos) = hand.iter().position(|e| e == card) {
                    hand.remove(pos);
                }
            }
        }

        self.broadcast_msg(self.get_msg(None, &state)?)?;
        self.broadcast_msg(self.room_state(&state))?;

        Ok(())
    }

    fn init_results(&self, state: &mut RwLockWriteGuard<RoomState>) -> Result<()> {
        state.stage = RoomStage::Results;

        let center_cards = self.get_center_cards(state);

        // choose random card to vote for if the player didn't choose
        for player in state.player_order.clone().iter() {
            if player != &state.player_order[state.active_player]
                && !state.player_to_vote.contains_key(player)
            {
                // choose random card
                let mut rng = rand::thread_rng();
                let mut card = center_cards.choose(&mut rng).unwrap().clone();

                // ensure player cannot choose their own card
                while &card == state.player_to_current_card.get(player).unwrap() {
                    card = center_cards.choose(&mut rng).unwrap().clone();
                }

                state.player_to_vote.insert(player.to_string(), card);
            }
        }

        let point_change = self.compute_results(state);

        // update with the point change
        state.players.iter_mut().for_each(|(player, info)| {
            if let Some(points) = point_change.get(player) {
                info.points += points;
            }
        });

        // send results to everyone
        self.broadcast_msg(self.get_msg(None, &state)?)?;
        self.broadcast_msg(self.room_state(&state))?;

        Ok(())
    }

    async fn init_round(&self, state: &mut RwLockWriteGuard<'_, RoomState>) -> Result<()> {
        if state.players.len() < 3 {
            return Err(anyhow!("Not enough players"));
        }

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

        state.deck = deck;
        state.player_hand = player_hand;
        state.stage = RoomStage::ActiveChooses;

        // notify players of the game start and their hands
        for player in state.player_order.iter() {
            let _ = self
                .send_msg(&state, &player, self.get_msg(Some(&player), &state)?)
                .await;
        }

        self.clear_ready(state);
        self.broadcast_msg(self.room_state(&state))?;

        Ok(())
    }

    pub async fn handle_client_msg(&self, name: &str, msg: WsMessage) -> Result<()> {
        let mut state = self.state.write().await;

        let msg: ClientMsg = serde_json::from_str(msg.to_text()?)
            .context(format!("Failed to deserialize client msg: {:?}", msg))?;

        println!("Handling client message: {:?}", msg);

        match msg {
            ClientMsg::Ready {} => {
                if matches!(state.stage, RoomStage::Joining)
                    || matches!(state.stage, RoomStage::Results)
                {
                    state
                        .players
                        .get_mut(name)
                        .ok_or_else(|| anyhow!("Unreachable: cannot ready player {}", name))?
                        .ready = true;

                    self.broadcast_msg(self.room_state(&state))?;

                    if state.players.values().filter(|p| p.ready).count() == state.players.len() {
                        self.init_round(&mut state).await?;
                    }
                }
            }
            ClientMsg::ActivePlayerChooseCard { card, description } => {
                if matches!(state.stage, RoomStage::ActiveChooses)
                    && state.player_order[state.active_player] == name
                {
                    // verify that player has this card
                    if !state.player_hand[name].contains(&card) {
                        return Err(anyhow!("Invalid card chosen by active player"));
                    }

                    let description = description.trim();
                    // verify that the description is not empty and is one word
                    if description.is_empty() || description.contains(' ') {
                        if let Some(tx) = state.player_to_socket.get(name) {
                            tx.send(
                                ServerMsg::ErrorMsg("Description must be one word".to_string())
                                    .into(),
                            )
                            .await?;
                        }
                        return Ok(());
                    }
                    state.current_description = description.to_string();
                    state.stage = RoomStage::PlayersChoose;

                    // record choice
                    state
                        .player_to_current_card
                        .insert(name.to_string(), card.to_string());

                    // notify players of the active player's choice
                    for player in state.player_order.iter() {
                        let _ = self
                            .send_msg(&state, &player, self.get_msg(Some(&player), &state)?)
                            .await;
                    }

                    self.clear_ready(&mut state);
                    self.broadcast_msg(self.room_state(&state))?;
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

                        // ready
                        state.players.get_mut(name).unwrap().ready = true;
                        self.broadcast_msg(self.room_state(&state))?;

                        // check if everyone except for the active player is ready
                        if state.players.values().filter(|p| p.ready).count()
                            == state.players.len() - 1
                        {
                            self.init_voting(&mut state)?;
                        }
                    }
                }
            }
            ClientMsg::Vote { card } => {
                if matches!(state.stage, RoomStage::Voting) {
                    // verify that the player is not the active player
                    if state.player_order[state.active_player] == name {
                        println!(
                            "{} is the active player",
                            state.player_order[state.active_player]
                        );
                        println!("{} is trying to vote", name);
                        return Err(anyhow!("Active player cannot vote"));
                    }

                    // verify that the card is in the center
                    if !state.player_to_current_card.values().any(|e| e == &card) {
                        return Err(anyhow!("Invalid card"));
                    }

                    // verify that this player is not voting for their own code or send an error message
                    if state.player_to_current_card.get(name).unwrap() == &card {
                        state
                            .player_to_socket
                            .get(name)
                            .unwrap()
                            .send(
                                ServerMsg::ErrorMsg(
                                    "You cannot vote for your own card".to_string(),
                                )
                                .into(),
                            )
                            .await?;
                        return Ok(());
                    }

                    // record vote
                    state
                        .player_to_vote
                        .insert(name.to_string(), card.to_string());

                    // ready
                    state.players.get_mut(name).unwrap().ready = true;
                    self.broadcast_msg(self.room_state(&state))?;

                    // check if everyone except for the active player is ready
                    if state.players.values().filter(|p| p.ready).count() == state.players.len() - 1
                    {
                        self.init_results(&mut state)?;
                    }
                }
            }
            _ => {
                // nothing
            }
        }

        Ok(())
    }

    fn compute_results(&self, state: &RwLockWriteGuard<RoomState>) -> HashMap<String, u16> {
        let mut point_change: HashMap<String, u16> = HashMap::new();
        let active_player = state.player_order[state.active_player].clone();
        let active_card = state
            .player_to_current_card
            .get(&active_player)
            .unwrap()
            .clone();

        let mut votes_for_card: HashMap<String, u16> = HashMap::new();

        for (_, card) in state.player_to_vote.iter() {
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
            }

            point_change.insert(active_player, 3);
        }

        for (player, card) in state.player_to_current_card.iter() {
            *point_change.get_mut(player).unwrap() += votes_for_card.get(card).unwrap_or(&0);
        }

        point_change
    }

    pub async fn on_connection(&self, socket: &mut WebSocket, name: &str) {
        // public funciton
        if let Err(e) = self.attempt_join(socket, name).await {
            println!("Error in attempt_join: {:?}", e);
            return;
        }

        let res = self.run_ws_loop(socket, name).await;
        println!("Player {} has left", name);

        let mut state = self.state.write().await;

        if matches!(state.stage, RoomStage::Joining) {
            state.players.remove(name);
        } else {
            if let Some(player) = state.players.get_mut(name) {
                player.connected = false;
            }
        }

        state.player_to_socket.remove(name);

        if let Err(e) = res {
            println!("Error in run_ws_loop: {:?}", e);
        }

        if let Err(e) = self.broadcast_msg(self.room_state(&state)) {
            println!("Error sending broadcast: {}", e);
        }
    }

    async fn attempt_join(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
        if name.is_empty() {
            socket
                .send(ServerMsg::ErrorMsg("Name cannot be empty".to_string()).into())
                .await?;
            return Err(anyhow!("Name cannot be empty"));
        }

        println!("Handling join for {}", name);

        let mut state = self.state.write().await;

        if let Some(player) = state.players.get_mut(name) {
            // player already exists in the game
            // and not in joining anymore
            // if in joining then player.active will be true

            if !player.connected {
                player.connected = true;
            } else {
                socket
                    .send(ServerMsg::ErrorMsg("Name already taken".to_string()).into())
                    .await?;
                return Err(anyhow!("Name already taken"));
            }
        } else if matches!(state.stage, RoomStage::Joining) {
            // still in joining and not yet joined
            if state.players.len() < 8 {
                state.players.insert(
                    name.to_string(),
                    PlayerInfo {
                        connected: true,
                        points: 0,
                        ready: false,
                    },
                );
            } else {
                socket
                    .send(ServerMsg::ErrorMsg("Too many players!".to_string()).into())
                    .await?;
                return Err(anyhow!("Too many players!"));
            }
        } else {
            socket
                .send(ServerMsg::ErrorMsg("Game has already started".to_string()).into())
                .await?;
            return Err(anyhow!("Game has already started"));
        }

        self.broadcast_msg(self.room_state(&state).into())?; // will not receive this one yet
        socket.send(self.room_state(&state).into()).await?;
        if let Ok(msg) = self.get_msg(Some(name), &state) {
            socket.send(msg.into()).await?;
        }

        Ok(())
    }

    async fn run_ws_loop(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
        println!("Starting loop for {}", name);

        let (tx, mut rx) = mpsc::channel(10);
        self.state
            .write()
            .await
            .player_to_socket
            .insert(name.to_string(), tx);
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
                    match msg {
                        Some(msg) => {
                            socket.send(msg.into()).await?;
                        }
                        _ => break,
                    }
                }
            }
        }

        Ok(())
    }

    fn broadcast_msg(&self, msg: ServerMsg) -> Result<()> {
        if self.broadcast.receiver_count() != 0 {
            self.broadcast.send(msg)?;
        }
        Ok(())
    }

    async fn send_msg(
        &self,
        state: &RwLockWriteGuard<'_, RoomState>,
        name: &str,
        msg: ServerMsg,
    ) -> Result<()> {
        let socket = state.player_to_socket.get(name).ok_or_else(|| {
            println!("Cannot find socket for {}", name);
            anyhow!("Cannot find socket for {}", name)
        })?;

        socket.send(msg.into()).await?;
        Ok(())
    }

    fn clear_ready(&self, state: &mut RwLockWriteGuard<RoomState>) {
        for (_, player) in state.players.iter_mut() {
            player.ready = false;
        }
    }

    pub async fn get_room_state(&self) -> ServerMsg {
        // TODO fix this shouldn't be a write
        let state = self.state.write().await;
        self.room_state(&state)
    }

    fn room_state(&self, state: &RwLockWriteGuard<RoomState>) -> ServerMsg {
        ServerMsg::RoomState {
            room_id: state.room_id.clone(),
            players: state.players.clone(),
            stage: state.stage,
            active_player: state.player_order.get(state.active_player).cloned(),
            player_order: state.player_order.clone(),
        }
    }
}
