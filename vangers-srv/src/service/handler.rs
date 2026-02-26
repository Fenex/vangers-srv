//! Tower `Service` implementation: handles `(ClientID, Packet)` and returns `Vec<ResponseAction>`.

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::future::Future;
use std::io::Write;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use tower::{BoxError, Service};
use tracing::{debug, info, warn};

use crate::client_id::ClientID;
use crate::game::{Type as GameType, World};
use crate::player::{Player, Status as PlayerStatus};
use crate::protocol::{Action, NetTransportSend, Packet};
use crate::server::SharedState;
use crate::utils::{get_first_cstr, slice_le_to_i16, slice_le_to_i32, slice_le_to_u32};
use crate::vanject::{NID, Vanject, VanjectError};

/// Who should receive the response packet.
#[derive(Debug, Clone)]
pub enum Target {
    /// Only the client that sent the request.
    Sender,
    /// All players in the same game except the sender.
    GameExceptSender,
    /// All players in the same game including the sender.
    AllInGame,
    /// Specific client IDs (e.g. for DIRECT_SENDING mask).
    Specific(Vec<ClientID>),
}

/// A single response to send: packet + destination.
#[derive(Debug, Clone)]
pub struct ResponseAction {
    pub target: Target,
    pub packet: Packet,
}

/// Inner service: handles one packet and returns the list of response actions.
pub struct VangersHandler {
    pub state: Arc<SharedState>,
}

impl VangersHandler {
    pub fn new(state: Arc<SharedState>) -> Self {
        Self { state }
    }
}

impl Service<(ClientID, Packet)> for VangersHandler {
    type Response = Vec<ResponseAction>;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Vec<ResponseAction>, BoxError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, (client_id, packet): (ClientID, Packet)) -> Self::Future {
        let state = Arc::clone(&self.state);

        Box::pin(async move {
            // Stub: dispatch to handlers that will be implemented in phase 6.
            // For now return "not implemented" for all actions.
            match packet.action {
                Action::ATTACH_TO_GAME => handle_attach_to_game(&state, client_id, &packet).await,
                Action::REGISTER_NAME => handle_register_name(&state, client_id, &packet).await,
                Action::SET_WORLD => handle_set_world(&state, client_id, &packet).await,
                Action::SET_GAME_DATA => handle_set_game_data(&state, client_id, &packet).await,
                Action::GET_GAME_DATA => handle_get_game_data(&state, client_id, &packet).await,
                Action::SET_PLAYER_DATA => handle_set_player_data(&state, client_id, &packet).await,
                Action::GAMES_LIST_QUERY => {
                    handle_games_list_query(&state, client_id, &packet).await
                }
                Action::SERVER_TIME_QUERY => {
                    handle_server_time_query(&state, client_id, &packet).await
                }
                Action::TOTAL_PLAYERS_DATA_QUERY => {
                    handle_total_players_data_query(&state, client_id, &packet).await
                }
                Action::CREATE_OBJECT => handle_create_object(&state, client_id, &packet).await,
                Action::UPDATE_OBJECT => handle_update_object(&state, client_id, &packet).await,
                Action::DELETE_OBJECT => handle_delete_object(&state, client_id, &packet).await,
                Action::DIRECT_SENDING => handle_direct_sending(&state, client_id, &packet).await,
                Action::LEAVE_WORLD => handle_leave_world(&state, client_id, &packet).await,
                Action::CLOSE_SOCKET => handle_close_socket(&state, client_id, &packet).await,
                _ => Err(format!("action {:?} not implemented", packet.action).into()),
            }
        })
    }
}

async fn handle_server_time_query(
    state: &SharedState,
    _client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let data = (state.uptime_secs() * 256).to_le_bytes();

    let packet: Packet = packet
        .create_answer(data.to_vec())
        .ok_or("SERVER_TIME_QUERY has no response type")?;

    Ok(vec![ResponseAction {
        target: Target::Sender,
        packet: packet,
    }])
}

async fn handle_games_list_query(
    state: &SharedState,
    _client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let mut games_count: u8 = 0;
    let mut data = vec![games_count];

    let games = state.games.read().await;
    for game in games.values() {
        if !game.is_configured() {
            continue;
        }

        let str_gmtype = match game.get_gmtype() {
            GameType::MECHOSOMA => 'M',
            GameType::VAN_WAR => 'V',
            GameType::PASSEMBLOSS => 'P',
            GameType::MIR_RAGE => 'R',
            _ => '?',
        };

        let title_head = String::from("[Rust-SRV] ");
        let title_tail = format!(
            ": {} {} {}",
            game.players.len(),
            str_gmtype,
            game.birth_time
        );

        let title = match game.name.last() {
            Some(0) => &game.name[..game.name.len() - 1],
            Some(_) => &game.name[..],
            None => b"[UNDEFINED TITLE]",
        };

        let mut name = std::iter::empty()
            .chain(CString::new(title_head).unwrap().as_bytes())
            .chain(title)
            .chain(CString::new(title_tail).unwrap().as_bytes())
            .chain(&[0])
            .copied()
            .collect::<Vec<_>>();

        data.append(&mut game.id.to_le_bytes().to_vec());
        data.append(&mut name);

        games_count += 1;
    }

    data[0] = games_count;

    let p = packet
        .create_answer(data)
        .ok_or("GAMES_LIST_QUERY has no response type")?;

    Ok(vec![ResponseAction {
        target: Target::Sender,
        packet: p,
    }])
}

async fn handle_attach_to_game(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    if packet.data.len() != 4 {
        Err("attach_to_game: required byte game_id not found")?
    }

    let mut games = state.games.write().await;

    let gmid = match slice_le_to_i32(&packet.data) {
        0 => {
            let gmid = state.next_game_id();
            games.create(gmid).ok();
            gmid as i32
        }
        gmid => gmid,
    };

    let game = games
        .get_mut_game_by_id(gmid as u32)
        .ok_or_else(|| format!("game with id `{}` not found", gmid))?;

    let player_id = game
        .attach_player(Player::new(client_id))
        .ok_or_else(|| format!("game {} has no free player slots", game.id))?;

    info!("attached player_id=`{player_id}` to game_id=`{gmid}`");

    let data = {
        // vanject ID offsets.
        // it is need to correct sync ID of vanjects if at least one player slot will
        // be free before. I don't know what actually algorithm do, but it just works

        let offsets = {
            let mut offsets = [0u16; 16];
            for (id, v) in &game.vanjects {
                let index = ((id >> 16) & 63) as usize;
                if v.get_station() == player_id as i32 && offsets[index] < (id & 0xFFFF) as u16 {
                    offsets[index] = (id & 0xFFFF) as u16;
                }
            }
            // it is possible to use `mem::transmute` or `byteorder` crate instead of creating new `Vec`
            offsets
                .iter()
                .flat_map(|&o| if o != 0 { o + 1 } else { o }.to_le_bytes().to_vec())
                .collect::<Vec<_>>()
        };

        // Game(4)
        // Configured(1) = 1 or 0
        // GameBirthTime(4)
        // Client_ID (player_id in rust)(1)
        // object_ID_offsets[16](short)
        std::iter::empty()
            .chain(&game.id.to_le_bytes())
            .chain(&(if game.is_configured() { 1u8 } else { 0u8 }).to_le_bytes())
            .chain(&(game.birth_time.as_secs_u32() as i32).to_le_bytes())
            .chain(&player_id.to_le_bytes())
            .chain(&offsets[..]) //object_ID_offsets
            .copied()
            .collect()
    };

    let mut actions = vec![];
    if let Some(p) = packet.create_answer(data) {
        actions.push(ResponseAction {
            target: Target::Sender,
            packet: p,
        });
    }

    let protocol = state.clients_protocol.read().await;
    if protocol.get(&client_id).copied().unwrap_or(1) > 1 {
        let unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        actions.push(ResponseAction {
            target: Target::Sender,
            packet: Packet::new(Action::Z_TIME_RESPONSE, &unix.to_le_bytes()),
        });
    }
    for v in game.vanjects.values() {
        actions.push(ResponseAction {
            target: Target::Sender,
            packet: Packet::new(Action::UPDATE_OBJECT, &v.to_vangers_byte()[..]),
        });
    }
    Ok(actions)
}

fn extract_auth_data(data: &[u8]) -> Result<(Cow<'_, CStr>, &CStr), BoxError> {
    let mut name = Cow::Borrowed(
        CStr::from_bytes_until_nul(data).map_err(|_| "name or password not C-string")?,
    );
    if name.is_empty() {
        return Err("request to set empty name".into());
    }
    if data.len() <= name.to_bytes_with_nul().len() {
        return Err("name or password parse".into());
    }
    let pwd = CStr::from_bytes_until_nul(&data[name.to_bytes_with_nul().len()..])
        .map_err(|_| "name or password parse")?;
    if name.count_bytes() > 16 {
        name = Cow::Owned(
            CString::from_vec_with_nul(
                name.to_bytes_with_nul()
                    .iter()
                    .take(15)
                    .chain(&[0])
                    .copied()
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| "name shrink")?,
        );
    }
    let name = if name.to_bytes().iter().any(|&c| c < 32 || c == 127) {
        Cow::Owned(
            CString::from_vec_with_nul(
                name.to_bytes_with_nul()
                    .iter()
                    .map(|&c| if c == 127 || (c > 0 && c < 32) { 42 } else { c })
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| "name filter")?,
        )
    } else {
        name
    };
    Ok((name, pwd))
}

async fn handle_register_name(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let mut games = state.games.write().await;
    let player = games
        .get_mut_player_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    let player_bind_id = player.bind.map(|b| b.id()).ok_or("player not bind")?;
    let (login, pwd) = extract_auth_data(&packet.data)?;
    player.set_auth(login.to_bytes_with_nul(), pwd.to_bytes_with_nul());
    info!("set name {:?} for player_id=`{}`", login, player_bind_id);
    let data: Vec<u8> = std::iter::empty()
        .chain(&player_bind_id.to_le_bytes())
        .chain(login.to_bytes_with_nul())
        .copied()
        .collect();
    let answer = packet
        .create_answer(data)
        .ok_or("REGISTER_NAME has no response")?;
    Ok(vec![ResponseAction {
        target: Target::GameExceptSender,
        packet: answer,
    }])
}

async fn handle_set_world(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let world_id = packet.data[0];
    let world_y_size = slice_le_to_i16(&packet.data[1..3]);
    let mut games = state.games.write().await;
    let game = games
        .get_mut_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    let mut world_status = 0u8;
    let world = if let Some(w) = game
        .worlds
        .iter()
        .find(|w| w.read().unwrap().id == world_id)
    {
        if w.read().unwrap().y_size != world_y_size {
            return Err(format!(
                "invalid world size: expected {:?}, given {}",
                w.read().unwrap().y_size,
                world_y_size
            )
            .into());
        }
        Arc::clone(&w)
    } else {
        world_status = 1;
        let w = Arc::new(RwLock::new(World::new(world_id, world_y_size)));
        game.worlds.push(Arc::clone(&w));
        w
    };
    let player = game.get_mut_player(client_id).unwrap();
    let player_bind_id = player.bind.map(|b| b.id()).ok_or("player not bind")?;
    let inventories_vanject: Vec<Packet> = game
        .vanjects
        .iter()
        .filter(|(_, v)| {
            v.get_type() != NID::VANGER
                && v.get_world() == world_id as i32
                && (!v.is_players() || v.is_non_global())
        })
        .map(|(_, v)| Packet::new(Action::UPDATE_OBJECT, &v.to_vangers_byte()))
        .collect();
    let world_guard = world.read().unwrap();
    let placed = game.place_player(client_id, &world_guard);
    drop(world_guard);
    let mut actions = vec![];
    if placed {
        actions.push(ResponseAction {
            target: Target::AllInGame,
            packet: Packet::new(
                Action::PLAYERS_STATUS,
                &[player_bind_id, PlayerStatus::GAMING as u8],
            ),
        });
    }
    actions.push(ResponseAction {
        target: Target::GameExceptSender,
        packet: Packet::new(Action::PLAYERS_WORLD, &[player_bind_id, world_id]),
    });
    actions.push(ResponseAction {
        target: Target::Sender,
        packet: packet
            .create_answer(vec![world_id, world_status])
            .ok_or("SET_WORLD has no response")?,
    });
    for p in inventories_vanject {
        actions.push(ResponseAction {
            target: Target::Sender,
            packet: p,
        });
    }
    Ok(actions)
}

async fn handle_set_game_data(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let mut games = state.games.write().await;
    let game = games
        .get_mut_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    if game.is_configured() {
        return Err(format!("game {} already configured", game.id).into());
    }
    let name = match CStr::from_bytes_until_nul(&packet.data) {
        Ok(n) if n.is_empty() => return Err("name is empty".into()),
        Ok(n) => n,
        Err(_) => return Err("parse name failed".into()),
    };
    game.set_config(&packet.data[name.to_bytes_with_nul().len()..])
        .map_err(|e| format!("config parse: {}", e))?;
    game.name = name.to_bytes_with_nul().to_vec();
    info!("changed config for game_id=`{}`", game.id);
    Ok(vec![])
}

async fn handle_get_game_data(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let games = state.games.read().await;
    let game = games
        .get_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    if !game.is_configured() {
        return Err(format!("game {} not configured", game.id).into());
    }
    let data: Vec<u8> = std::iter::empty()
        .chain(&game.name)
        .chain(&game.config.as_ref().unwrap().to_vangers_byte())
        .copied()
        .collect();
    let p = packet
        .create_answer(data)
        .ok_or("GET_GAME_DATA has no response")?;
    Ok(vec![ResponseAction {
        target: Target::Sender,
        packet: p,
    }])
}

async fn handle_set_player_data(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let mut games = state.games.write().await;
    let game = games
        .get_mut_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    let player = game.get_mut_player(client_id).unwrap();
    let player_id = match (player.set_body(&packet.data), player.bind) {
        (Ok(_), Some(bind)) => bind.id(),
        (Ok(_), None) => return Err("player not bind".into()),
        (Err(_), _) => return Err("set_body failed".into()),
    };
    let data: Vec<u8> = std::iter::empty()
        .chain(&[player_id])
        .chain(&packet.data)
        .copied()
        .collect();
    let p = packet
        .create_answer(data)
        .ok_or("SET_PLAYER_DATA has no response")?;
    Ok(vec![ResponseAction {
        target: Target::GameExceptSender,
        packet: p,
    }])
}

async fn handle_total_players_data_query(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let games = state.games.read().await;
    let game = games
        .get_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    let mut data = vec![game.players.len() as u8];
    let mut players_count = 0u8;
    for player in &game.players {
        let id = match player.bind {
            Some(bind) => bind.id(),
            None => continue,
        };
        let status = player.status as u8;
        let world = match &player.world {
            Some(w) => w.read().unwrap().id,
            None => 0u8,
        };
        let name = match &player.auth {
            Some(auth) => auth.name(),
            None => b"[UNDEFINED]\0",
        };
        let body = match &player.body {
            Some(b) => b.to_vangers_byte(),
            None => {
                warn!(
                    "player (bind_id={} client_id={}) has no body, ignored",
                    id, player.client_id
                );
                continue;
            }
        };
        let mut p_data: Vec<u8> = std::iter::empty()
            .chain(&[id])
            .chain(&[status])
            .chain(&[world])
            .chain(&player.pos.to_vangers_byte())
            .chain(name)
            .chain(&body)
            .copied()
            .collect();
        data.append(&mut p_data);
        players_count += 1;
    }
    data[0] = players_count;
    let p = packet
        .create_answer(data)
        .ok_or("TOTAL_PLAYERS_DATA_QUERY has no response")?;
    Ok(vec![ResponseAction {
        target: Target::Sender,
        packet: p,
    }])
}

async fn handle_create_object(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let mut vanject = Vanject::create_from_slice(&packet.data)
        .map_err(|e: VanjectError| format!("vanject parse: {:?}", e))?;
    let mut games = state.games.write().await;
    let game = games
        .get_mut_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    let mut actions = vec![];
    if game.vanjects.contains_key(&vanject.id) {
        debug!("VANJECT with id=`{}` already exists", vanject.id);
        return Ok(actions);
    }
    let player = game.get_mut_player(client_id).unwrap();
    if vanject.bind_to_player(player).is_err() {
        return Err("player not bind".into());
    }
    if vanject.get_type() == NID::VANGER {
        player.pos = vanject.pos;
        if player.set_body(&vanject.body).is_err() {
            warn!("NID::VANGER: set body failed");
        } else {
            let data = vanject.to_vangers_byte();
            actions.push(ResponseAction {
                target: Target::GameExceptSender,
                packet: Packet::new(Action::UPDATE_OBJECT, &data),
            });
            let pos_data: Vec<u8> = std::iter::empty()
                .chain(&[vanject.player_bind_id])
                .chain(&vanject.pos.to_vangers_byte())
                .copied()
                .collect();
            actions.push(ResponseAction {
                target: Target::GameExceptSender,
                packet: Packet::new(Action::PLAYERS_POSITION, &pos_data),
            });
        }
    } else {
        let data = vanject.to_vangers_byte();
        let p = Packet::new(Action::UPDATE_OBJECT, &data);
        if !vanject.is_players() {
            actions.push(ResponseAction {
                target: Target::GameExceptSender,
                packet: p,
            });
        } else if vanject.is_non_global() {
            debug!(
                "Added vanject {:?} to inventory of player_id=`{}`",
                &vanject.id.to_le_bytes(),
                vanject.player_bind_id
            );
            actions.push(ResponseAction {
                target: Target::GameExceptSender,
                packet: p,
            });
        } else {
            actions.push(ResponseAction {
                target: Target::GameExceptSender,
                packet: p,
            });
        }
    }
    game.vanjects.insert(vanject.id, vanject);
    Ok(actions)
}

async fn handle_update_object(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    if packet.data.len() < 4 {
        return Err("update_object: slice too small".into());
    }
    let vanject_id = slice_le_to_i32(&packet.data[0..4]);
    let mut games = state.games.write().await;
    let game = games
        .get_mut_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    let player_bind_id = game
        .get_player(client_id)
        .and_then(|p| p.bind.map(|b| b.id()))
        .ok_or("player not bind")?;
    let vanject = game
        .vanjects
        .get_mut(&vanject_id)
        .ok_or_else(|| format!("vanject id={} not found", vanject_id))?;
    vanject
        .update_from_slice(&packet.data)
        .map_err(|e| format!("{:?}", e))?;
    vanject.player_bind_id = player_bind_id;
    let mut actions = vec![];
    if vanject.get_type() == NID::VANGER {
        let data: Vec<u8> = std::iter::empty()
            .chain(&[vanject.player_bind_id])
            .chain(&vanject.pos.to_vangers_byte())
            .copied()
            .collect();
        actions.push(ResponseAction {
            target: Target::GameExceptSender,
            packet: Packet::new(Action::PLAYERS_POSITION, &data),
        });
    }
    actions.push(ResponseAction {
        target: Target::GameExceptSender,
        packet: Packet::new(Action::UPDATE_OBJECT, &vanject.to_vangers_byte()),
    });
    Ok(actions)
}

async fn handle_delete_object(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    if packet.data.len() < 8 {
        return Err("delete_object: data too small".into());
    }
    let vanject_id = slice_le_to_i32(&packet.data[0..4]);
    let mut games = state.games.write().await;
    let game = games
        .get_mut_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    let player_auth_id = game
        .get_mut_player(client_id)
        .and_then(|p| p.bind.map(|b| b.id()))
        .ok_or("player not bind")?;
    let data: Vec<u8> = std::iter::empty()
        .chain(&vanject_id.to_le_bytes())
        .chain(&[player_auth_id])
        .chain(&packet.data[4..8])
        .chain(&packet.data[8..])
        .copied()
        .collect();
    let answer = Packet::new(Action::DELETE_OBJECT, &data);
    game.vanjects.remove(&vanject_id);
    Ok(vec![ResponseAction {
        target: Target::GameExceptSender,
        packet: answer,
    }])
}

async fn handle_direct_sending(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    if packet.data.len() < 4 + 1 + 1 {
        return Err("direct_sending: data too small".into());
    }
    let games = state.games.read().await;
    let game = games
        .get_game_by_client_id(client_id)
        .ok_or_else(|| format!("client {} not in any game", client_id))?;
    let mask = slice_le_to_u32(&packet.data[0..4]);
    let mut player_opt: Option<&Player> = None;
    let mut client_ids = vec![];
    for p in &game.players {
        if let Some(bind) = p.bind {
            if p.client_id == client_id {
                player_opt = Some(p);
            } else if (bind.mask() as u32) & mask != 0 {
                client_ids.push(p.client_id);
            }
        }
    }
    let player = player_opt.ok_or_else(|| format!("player {} not found in game", client_id))?;
    if player.bind.is_none() || player.auth.is_none() {
        return Err("player not bind".into());
    }
    let player_id = player.bind.unwrap().id();
    let msg = get_first_cstr(&packet.data[4..]).ok_or("direct_sending: cannot parse c-string")?;
    let mut msg = Cow::Borrowed(msg);
    const LIMIT_MSG_LEN: usize = 140;
    if msg.len() > LIMIT_MSG_LEN {
        warn!("direct message too long, truncating");
        let mut buffer = Vec::with_capacity(LIMIT_MSG_LEN);
        let _ = buffer.write(&msg[0..LIMIT_MSG_LEN.saturating_sub(4)]);
        let _ = buffer.write(b"...");
        buffer.push(0);
        msg = Cow::Owned(buffer);
    }
    let data: Vec<u8> = std::iter::empty()
        .chain(&[player_id])
        .chain(&msg[..])
        .copied()
        .collect();
    let answer = packet
        .create_answer(data)
        .ok_or("DIRECT_SENDING has no response")?;
    Ok(vec![ResponseAction {
        target: Target::Specific(client_ids),
        packet: answer,
    }])
}

async fn handle_leave_world(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let mut games = state.games.write().await;
    let game = games
        .get_mut_game_by_client_id(client_id)
        .ok_or_else(|| format!("player client_id={} not found", client_id))?;
    let player = game.get_mut_player(client_id).unwrap();
    let player_bind_id = player.bind.map(|b| b.id()).ok_or("player not bind")?;
    let _world_id = match &player.world {
        Some(w) => w.read().unwrap().id,
        None => return Err("player is out of all worlds".into()),
    };
    player.world = None;
    let mut actions = vec![];
    let delete: Vec<_> = game
        .vanjects
        .iter()
        .filter(|(_, v)| v.get_station() == player_bind_id as i32 && v.is_private())
        .map(|(&id, v)| {
            (
                id,
                Packet::new(
                    Action::DELETE_OBJECT,
                    &std::iter::empty()
                        .chain(&id.to_le_bytes())
                        .chain(&[player_bind_id])
                        .chain(&v.time.to_le_bytes())
                        .copied()
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect();
    for (id, p) in &delete {
        actions.push(ResponseAction {
            target: Target::GameExceptSender,
            packet: p.clone(),
        });
        game.vanjects.remove(id);
    }
    actions.push(ResponseAction {
        target: Target::GameExceptSender,
        packet: Packet::new(Action::PLAYERS_WORLD, &[player_bind_id, 0u8]),
    });
    Ok(actions)
}

async fn handle_close_socket(
    state: &SharedState,
    client_id: ClientID,
    packet: &Packet,
) -> Result<Vec<ResponseAction>, BoxError> {
    let mut actions = handle_leave_world(state, client_id, packet)
        .await
        .unwrap_or_default();
    let (game_id, remove_game) = {
        let mut games = state.games.write().await;
        let game = match games.get_mut_game_by_client_id(client_id) {
            Some(g) => g,
            None => return Err(format!("player client_id={} not found", client_id).into()),
        };
        let player = game.get_mut_player(client_id).unwrap();
        let player_bind_id = player.bind.map(|b| b.id()).ok_or("player not bind")?;
        player.world = None;
        if player.status == PlayerStatus::GAMING {
            player.status = PlayerStatus::FINISHED;
            actions.push(ResponseAction {
                target: Target::GameExceptSender,
                packet: Packet::new(
                    Action::PLAYERS_STATUS,
                    &[player_bind_id, PlayerStatus::FINISHED as u8],
                ),
            });
        }
        game.players.retain(|p| p.client_id != client_id);
        let game_id = game.id;
        let remove_game = game.players.is_empty();
        (game_id, remove_game)
    };
    if remove_game {
        state.games.write().await.remove(&game_id);
    }
    Ok(actions)
}

/// Clean up game state when a client disconnects (e.g. connection closed).
pub async fn handle_disconnect(state: &SharedState, client_id: ClientID) {
    let _ = handle_close_socket(state, client_id, &Packet::new(Action::CLOSE_SOCKET, &[])).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Config, Game, Type as GameType};
    use crate::player::{Body as PlayerBody, Player};

    #[tokio::test]
    async fn handler_games_list_query_empty() {
        let state = Arc::new(SharedState::new());
        let mut svc = VangersHandler::new(state);
        let packet = Packet::new(Action::GAMES_LIST_QUERY, &[]);
        let result = svc.call((1, packet)).await;
        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].packet.action, Action::GAMES_LIST_RESPONSE);
        assert_eq!(actions[0].packet.data, &[0]);
    }

    #[tokio::test]
    async fn handler_games_list_query_configured_game() {
        let state = Arc::new(SharedState::new());
        state.games.write().await.insert(1, {
            let mut g = Game::new(1);
            g.config = Some(Config::new(GameType::PASSEMBLOSS));
            let mut p = Player::new(11);
            p.set_auth(b"player\0", b"\0");
            p.body = Some(PlayerBody::default());
            g.attach_player(p);
            g
        });
        let mut svc = VangersHandler::new(state);
        let packet = Packet::new(Action::GAMES_LIST_QUERY, &[]);
        let result = svc.call((1, packet)).await;
        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].packet.action, Action::GAMES_LIST_RESPONSE);
        assert_eq!(actions[0].packet.data[0], 1);
        assert_eq!(actions[0].packet.data[1..5], 1u32.to_le_bytes());
    }
}
