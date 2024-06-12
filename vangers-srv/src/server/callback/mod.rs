mod attach_to_game;
mod close_socket;
mod create_object;
mod delete_object;
mod direct_sending;
mod games_list_query;
mod get_game_data;
mod leave_world;
mod register_name;
mod server_time_query;
mod set_game_data;
mod set_player_data;
mod set_world;
mod total_players_data_query;
mod update_object;

use attach_to_game::*;
pub(crate) use close_socket::*;
use create_object::*;
use delete_object::*;
use direct_sending::*;
use games_list_query::*;
use get_game_data::*;
use leave_world::*;
use register_name::*;
use server_time_query::*;
use set_game_data::*;
use set_player_data::*;
use set_world::*;
use total_players_data_query::*;
use update_object::*;

use ::log::{trace, debug, info, warn, error};

use crate::client::ClientID;
use crate::protocol::{Action, Packet};
use crate::Server;

#[derive(Debug, ::thiserror::Error)]
pub enum OnUpdateError {
    // #[error("Handled Debug for: {:?}, Packet: {:?}")]
    // DebugError((Action, Vec<u8>)),
    #[error("{}", not_implemented_errdisplay(.0))]
    NotImplementedAction(Packet),
    #[error("response type for action {0:?} is not exists")]
    ResponsePacketTypeNotExist(Action),
    #[error("AttachToGameError: {0}")]
    AttachToGameError(#[from] AttachToGameError),
    #[error("RegisterNameError: {0}")]
    RegisterNameError(#[from] RegisterNameError),
    #[error("SetPlayerDataError: {0}")]
    SetPlayerDataError(#[from] SetPlayerDataError),
    #[error("SetGameDataError: {0}")]
    SetGameDataError(#[from] SetGameDataError),
    #[error("GetGameDataError: {0}")]
    GetGameDataError(#[from] GetGameDataError),
    #[error("CreateObjectError: {0}")]
    CreateObjectError(#[from] CreateObjectError),
    #[error("UpdateObjectError: {0}")]
    UpdateObjectError(#[from] UpdateObjectError),
    #[error("DeleteObjectError: {0}")]
    DeleteObjectError(#[from] DeleteObjectError),
    #[error("DirectSendingError: {0}")]
    DirectSendingError(#[from] DirectSendingError),
    #[error("TotalPlayersDataQueryError: {0}")]
    TotalPlayersDataQueryError(#[from] TotalPlayersDataQueryError),
    #[error("SetWorldError: {0}")]
    SetWorldError(#[from] SetWorldError),
    #[error("LeaveWorldError: {0}")]
    LeaveWorldError(#[from] LeaveWorldError),
    #[error("CloseSocketError: {0}")]
    CloseSocketError(#[from] CloseSocketError),
}

fn not_implemented_errdisplay(packet: &Packet) -> String {
    let mut out = format!("action {:?} is not implemented", &packet.action);
    if packet.action == Action::UNKNOWN {
        out.push_str(&format!("{}({}): {:?}", out, packet.real_action, packet.as_bytes()));
    }
    out
}

#[derive(Debug)]
pub enum OnUpdateOk {
    #[allow(dead_code)]
    Broadcast(Packet),
    Response(Packet),
    Complete,
}

pub(super) trait OnUpdate {
    fn on_update(&mut self, client_id: ClientID, packet: Packet);
}

impl OnUpdate for Server {
    fn on_update(&mut self, client_id: ClientID, packet: Packet) {

        view ("[<-]", &packet);

        let result = match packet.action {
            Action::ATTACH_TO_GAME => self.attach_to_game(&packet, client_id),
            Action::SERVER_TIME_QUERY => self.server_time_query(&packet, client_id),
            Action::GAMES_LIST_QUERY => self.games_list_query(&packet, client_id),
            Action::TOTAL_PLAYERS_DATA_QUERY => self.total_players_data_query(&packet, client_id),
            Action::REGISTER_NAME => self.register_name(&packet, client_id),
            Action::SET_PLAYER_DATA => self.set_player_data(&packet, client_id),
            Action::SET_GAME_DATA => self.set_game_data(&packet, client_id),
            Action::GET_GAME_DATA => self.get_game_data(&packet, client_id),
            Action::CREATE_OBJECT => self.create_object(&packet, client_id),
            Action::SET_WORLD => self.set_world(&packet, client_id),
            Action::LEAVE_WORLD => self.leave_world(&packet, client_id),
            Action::UPDATE_OBJECT => self.update_object(&packet, client_id),
            Action::DELETE_OBJECT => self.delete_object(&packet, client_id),
            Action::DIRECT_SENDING => self.direct_sending(&packet, client_id),
            Action::CLOSE_SOCKET => self.close_socket(&packet, client_id),
            _ => Err(OnUpdateError::NotImplementedAction(packet.clone())),
        };

        match result {
            Ok(OnUpdateOk::Response(p)) => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
                    view("[->]", &packet);
                    client.send(&p);
                } else {
                    warn!("Error: can't send response to client with id={}", client_id);
                }
            }
            Ok(OnUpdateOk::Broadcast(p)) => {
                view("[=>]", &packet);
                self.notify_all(client_id, &p);
            }
            Ok(OnUpdateOk::Complete) => {
                match &packet.action {
                    Action::CREATE_OBJECT | Action::UPDATE_OBJECT | Action::DELETE_OBJECT => {}
                    _ => view("[ok]", &packet),
                };
            }
            Err(err) => error!("{}", err),
        }
    }
}

fn view(prefix: &str, p: &Packet) {
    use Action::*;

    match &p.action {
        a @ (CREATE_OBJECT | UPDATE_OBJECT | DELETE_OBJECT) => {
            trace!("{} {:?}: {:X?}", prefix, a, &p.data);
        },
        a @ (SERVER_TIME | SERVER_TIME_QUERY | SERVER_TIME_RESPONSE) => {
            trace!("{} {:?}: {:X?}", prefix, a, &p.data);
        },
        a @ GAMES_LIST_QUERY => debug!("{} {:?}", prefix, a),
        a => {
            debug!("{} {:?}: {:?}", prefix, a, &p.data);
            info!("{} {:?}", prefix, a);
        },
    }
}
