use crate::player::Status as PlayerStatus;
use crate::protocol::{Action, Packet};
// use crate::vanject::{VanjectError};
use crate::Server;
use crate::client::ClientID;

use super::{LeaveWorldError, OnUpdate_LeaveWorld, OnUpdateError, OnUpdateOk};

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
        match self.leave_world(packet, client_id) {
            Ok(_) => {}
            Err(OnUpdateError::LeaveWorldError(LeaveWorldError::WorldEmpty(_))) => {}
            Err(err) => return Err(err),
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, World};
    use crate::player::Player;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn close_socket_propagates_leave_world_failures() {
        let mut srv = Server::new(Default::default());
        let mut game = Game::new(1);
        let client_id: ClientID = 11;
        game.attach_player(Player::new(client_id));

        let world = Rc::new(RefCell::new(World::new(1, 100)));
        game.worlds.push(Rc::clone(&world));
        game.place_player(client_id, &world.borrow());

        srv.games.insert(1, game);

        let _borrow_guard = world.borrow_mut();
        let err = srv
            .close_socket(&Packet::new(Action::CLOSE_SOCKET, &[]), client_id)
            .unwrap_err();

        match err {
            OnUpdateError::LeaveWorldError(LeaveWorldError::BorrowWorld(id, _)) => {
                assert_eq!(id, client_id);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
