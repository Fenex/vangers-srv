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

use std::fmt;

use crate::client::ClientID;
use crate::protocol::{Action, Packet};
use crate::Server;

#[derive(Debug)]
pub enum OnUpdateError {
    #[allow(dead_code)]
    DebugError((Action, Vec<u8>)),
    NotImplementedAction(Packet),
    ResponsePacketTypeNotExist(Action),
    AttachToGameError(AttachToGameError),
    // ServerTimeQueryError(ServerTimeQueryError),
    RegisterNameError(RegisterNameError),
    SetPlayerDataError(SetPlayerDataError),
    SetGameDataError(SetGameDataError),
    GetGameDataError(GetGameDataError),
    CreateObjectError(CreateObjectError),
    UpdateObjectError(UpdateObjectError),
    DeleteObjectError(DeleteObjectError),
    DirectSendingError(DirectSendingError),
    TotalPlayersDataQueryError(TotalPlayersDataQueryError),
    SetWorldError(SetWorldError),
    LeaveWorldError(LeaveWorldError),
    CloseSocketError(CloseSocketError),
}

impl fmt::Display for OnUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DebugError((a, d)) => write!(f, "Handled Debug for: {:?}, Packet: {:?}", a, d),
            Self::NotImplementedAction(packet) => {
                let mut out = format!("action {:?} is not implemented", &packet.action);
                if packet.action == Action::UNKNOWN {
                    out = format!("{}({}): {:?}", out, packet.real_action, packet.as_bytes());
                }
                write!(f, "{}", out)
            }
            Self::ResponsePacketTypeNotExist(a) => {
                write!(f, "response type for action {:?} is not exists", a)
            }
            Self::AttachToGameError(e) => write!(f, "AttachToGameError: {}", e),
            Self::RegisterNameError(e) => write!(f, "RegisterNameError: {}", e),
            Self::SetPlayerDataError(e) => write!(f, "SetPlayerDataError: {}", e),
            Self::SetGameDataError(e) => write!(f, "SetGameDataError: {}", e),
            Self::GetGameDataError(e) => write!(f, "GetGameDataError: {}", e),
            Self::CreateObjectError(e) => write!(f, "CreateObjectError: {}", e),
            Self::UpdateObjectError(e) => write!(f, "UpdateObjectError: {}", e),
            Self::DeleteObjectError(e) => write!(f, "DeleteObjectError: {}", e),
            Self::DirectSendingError(e) => write!(f, "DirectSendingError: {}", e),
            Self::TotalPlayersDataQueryError(e) => write!(f, "TotalPlayersDataQueryError: {}", e),
            Self::SetWorldError(e) => write!(f, "SetWorldError: {}", e),
            Self::LeaveWorldError(e) => write!(f, "LeaveWorldError: {}", e),
            Self::CloseSocketError(e) => write!(f, "CloseSocketError: {}", e),
        }
    }
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
        match &packet.action {
            Action::SERVER_TIME_QUERY
            | Action::CREATE_OBJECT
            | Action::DELETE_OBJECT
            | Action::UPDATE_OBJECT => {
                // println!("[<-] {:?}: {:?}", packet.action, &packet.data);
            }
            a => println!("[<-] {:?}", a),
        }

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
                    println!("Error: can't send response to client with id={}", client_id);
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
            Err(err) => println!("{}", err),
        }
    }
}

fn view(prefix: &str, p: &Packet) {
    match &p.action {
        Action::CREATE_OBJECT | Action::UPDATE_OBJECT | Action::DELETE_OBJECT => (),
        Action::SERVER_TIME | Action::SERVER_TIME_QUERY | Action::SERVER_TIME_RESPONSE => (),
        Action::GAMES_LIST_QUERY => (),
        a => println!("{} {:?}", prefix, a),
    }
}
