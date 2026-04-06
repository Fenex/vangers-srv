use std::{cell::BorrowError, collections::HashMap};

use crate::Server;
use crate::client::ClientID;
use crate::protocol::{Action, Packet};

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum LeaveWorldError {
    #[error("player with client_id `{0}` not found")]
    PlayerNotFound(ClientID),
    #[error("player with client_id `{0}` not bind")]
    PlayerNotBind(ClientID),
    #[error("player with client_id `{0}` is out of all worlds")]
    WorldEmpty(ClientID),
    #[error("cannot get player's world (client_id `{0}`): {1}")]
    BorrowWorld(ClientID, BorrowError),
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_LeaveWorld {
    fn leave_world(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_LeaveWorld for Server {
    #[tracing::instrument(skip_all)]
    fn leave_world(
        &mut self,
        _: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let game = match self.get_mut_game_by_clientid(client_id) {
            Some(game) => game,
            None => return Err(LeaveWorldError::PlayerNotFound(client_id).into()),
        };

        let player = game.get_mut_player(client_id).unwrap();
        let player_bind_id = match player.bind {
            Some(bind) => bind.id(),
            None => return Err(LeaveWorldError::PlayerNotBind(client_id).into()),
        };

        let _world_id = match player.world {
            Some(ref world) => match world.try_borrow() {
                Ok(w) => w.id,
                Err(e) => return Err(LeaveWorldError::BorrowWorld(client_id, e).into()),
            },
            None => return Err(LeaveWorldError::WorldEmpty(client_id).into()),
        };
        player.world = None;

        let delete = game
            .vanjects
            .iter()
            .filter(|(_, v)| v.get_station() == player_bind_id as i32 && v.is_non_global())
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
            .collect::<HashMap<_, _>>();

        game.vanjects.retain(|id, _| !delete.contains_key(id));

        for (_, packet) in delete {
            self.notify_game(client_id, &packet);
        }

        // That sends by github server, but it seems to may be safety removed at all
        self.notify_game(
            client_id,
            &Packet::new(Action::PLAYERS_WORLD, &[player_bind_id, 0u8]),
        );

        Ok(OnUpdateOk::Complete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, World};
    use crate::player::Player;
    use crate::vanject::Vanject;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn leave_world_removes_player_owned_non_private_objects_too() {
        let mut srv = Server::new(Default::default());
        let mut game = Game::new(1);
        let client_id: ClientID = 11;
        game.attach_player(Player::new(client_id));

        let world = Rc::new(RefCell::new(World::new(1, 100)));
        game.worlds.push(Rc::clone(&world));
        game.place_player(client_id, &world.borrow());

        let mut slot = Vanject::create_from_slice(&[
            1, 0, 2, 4, // player-owned SLOT object, non-private but non-global
            6, 0, 0, 0, // time
            10, 0, // x
            20, 0, // y
            15, 0, // radius
            1, 2, 3,
        ])
        .unwrap();
        slot.player_bind_id = 1;
        let slot_id = slot.id;
        game.vanjects.insert(slot_id, slot);

        srv.games.insert(1, game);
        srv.leave_world(&Packet::new(Action::LEAVE_WORLD, &[]), client_id)
            .unwrap();

        let game = srv.games.get(&1).unwrap();
        assert!(!game.vanjects.contains_key(&slot_id));
    }
}
