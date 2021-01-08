use std::fmt;

use crate::player::Status as PlayerStatus;
use crate::protocol::{Action, Packet};
// use crate::vanject::{VanjectError};
use crate::client::ClientID;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk, OnUpdate_LeaveWorld};

#[derive(Debug)]
pub enum CloseSocketError {
    SliceTooSmall,
    // SliceToVanjectParse(VanjectError),
    PlayerNotFound(ClientID),
    // VanjectNotFound(i32),
    PlayerNotBind(ClientID),
}

impl fmt::Display for CloseSocketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SliceTooSmall => write!(f, "fail read slice as vanject: [too small slice]"),
            // Self::SliceToVanjectParse(e) => write!(f, "fail read slice as vanject: [{}]", e),
            Self::PlayerNotFound(id) => write!(f, "player with `client_id`={} not found", id),
            // Self::VanjectNotFound(id) => write!(f, "vanject with `id`={} not found", id),
            Self::PlayerNotBind(id) => write!(f, "player with `client_id`={} not bind", id),
        }
    }
}

impl From<CloseSocketError> for OnUpdateError {
    fn from(from: CloseSocketError) -> Self {
        Self::CloseSocketError(from)
    }
}

#[allow(non_camel_case_types)]
pub(in crate::server) trait OnUpdate_CloseSocket {
    fn close_socket(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_CloseSocket for Server {
    fn close_socket(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        self.leave_world(packet, client_id).ok();

        // TODO: needs to free a player slot
        // self.free_player_slot();

        let game = match self.get_mut_game_by_clientid(client_id) {
            Some(game) => game,
            None => return Err(CloseSocketError::PlayerNotFound(client_id).into()),
        };

        let player = game.get_mut_player(client_id).unwrap();
        let player_bind_id = match player.bind {
            Some(bind) => bind.id(),
            None => return Err(CloseSocketError::PlayerNotBind(client_id).into()),
        };

        player.world = None;
        if player.status == PlayerStatus::GAMING {
            player.status = PlayerStatus::FINISHED;
            self.notify_game(
                client_id,
                &Packet::new(
                    Action::PLAYERS_STATUS,
                    &[player_bind_id, PlayerStatus::FINISHED as u8],
                ),
            );
        }

        let game = self.get_mut_game_by_clientid(client_id).unwrap();

        game.players.retain(|p| p.client_id != client_id);
        // TODO: recalc game ratings
        // self.process_ratings(game.gmtype);

        Ok(OnUpdateOk::Complete)
    }
}
