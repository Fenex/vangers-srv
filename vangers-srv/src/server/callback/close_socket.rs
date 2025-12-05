use crate::player::Status as PlayerStatus;
use crate::protocol::{Action, Packet};
// use crate::vanject::{VanjectError};
use crate::client::ClientID;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk, OnUpdate_LeaveWorld};

#[derive(Debug, ::thiserror::Error)]
pub enum CloseSocketError {
    // #[error("fail read slice as vanject: [too small slice]")]
    // SliceTooSmall,
    // SliceToVanjectParse(VanjectError),
    #[error("player with `client_id`={0} not found")]
    PlayerNotFound(ClientID),
    // VanjectNotFound(i32),
    #[error("player with `client_id`={0} not bind")]
    PlayerNotBind(ClientID),
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
    #[tracing::instrument(skip_all)]
    fn close_socket(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        self.leave_world(packet, client_id).ok();

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

        if game.players.is_empty() {
            let game_id = game.id;
            self.games.remove(&game_id);
        }

        Ok(OnUpdateOk::Complete)
    }
}
