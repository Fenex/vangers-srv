use std::cell::RefCell;
use std::rc::Rc;

use crate::Server;
use crate::client::ClientID;
use crate::game::World;
use crate::player::Status as PlayerStatus;
use crate::protocol::{Action, Packet};
use crate::utils::slice_le_to_i16;
use crate::vanject::NID;

use super::{OnUpdate_LeaveWorld, OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum SetWorldError {
    #[error("player with client_id `{0}` not found")]
    PlayerNotFound(ClientID),
    #[error("player with client_id `{0}` not bind")]
    PlayerNotBind(ClientID),
    #[error("invalid world size: expected `{0}`, given `{1}`")]
    InvalidWorldSize(i16, i16),
    // #[error("fail read data slice")]
    // DataParse,
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_SetWorld {
    fn set_world(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_SetWorld for Server {
    #[tracing::instrument(skip_all)]
    fn set_world(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let world_id = packet.data[0];
        let world_y_size = slice_le_to_i16(&packet.data[1..3]);

        let needs_leave_world = self
            .get_game_by_clientid(client_id)
            .and_then(|game| game.get_player(client_id))
            .and_then(|player| player.world.as_ref())
            .map(|world| world.borrow().id != world_id)
            .unwrap_or(false);

        if needs_leave_world {
            self.leave_world(&Packet::new(Action::LEAVE_WORLD, &[]), client_id)?;
        }

        let game = self
            .get_mut_game_by_clientid(client_id)
            .ok_or(SetWorldError::PlayerNotFound(client_id))?;

        // Must be sets to `1` if new world was created
        let mut world_status = 0u8;

        // TODO: take out below if-else code into Game struct
        let world = if let Some(world) = game.worlds.iter().find(|w| w.borrow().id == world_id) {
            if world.borrow().y_size != world_y_size {
                Err(SetWorldError::InvalidWorldSize(
                    world.borrow().y_size,
                    world_y_size,
                ))?
            }
            Rc::clone(world)
        } else {
            // create new world with `world_id`
            let world = Rc::new(RefCell::new(World::new(world_id, world_y_size)));
            game.worlds.push(Rc::clone(&world));
            // game.place_player(client_id, &world.borrow());

            world_status = 1;

            Rc::clone(&world)
        };

        let player = game.get_mut_player(client_id).unwrap();
        let player_bind_id = player
            .bind
            .map(|bind| bind.id())
            .ok_or(SetWorldError::PlayerNotBind(client_id))?;

        // get all dropped items and items inside inventories of all players in the current world
        let inventories_vanject = game
            .vanjects
            .iter()
            .filter(|(_, v)| {
                v.get_type() != NID::VANGER
                    && v.get_world() == world_id as i32
                    // && v.get_station() != player_bind_id as i32 // it is REAL not need con
                    && (!v.is_players() || v.is_players() && v.is_non_global())
            })
            .map(|(_, v)| Packet::new(Action::UPDATE_OBJECT, &v.to_vangers_byte()))
            .collect::<Vec<_>>();

        if game.place_player(client_id, &world.borrow()) {
            let packet = Packet::new(
                Action::PLAYERS_STATUS,
                &[player_bind_id, PlayerStatus::GAMING as u8],
            );
            self.notify_all(client_id, &packet);
        }

        let answer = Packet::new(Action::PLAYERS_WORLD, &[player_bind_id, world_id]);
        self.notify_game(client_id, &answer);

        let answer = packet.create_answer(vec![world_id, world_status]).unwrap();
        self.notify_player(client_id, &answer);

        inventories_vanject
            .iter()
            .for_each(|p| self.notify_player(client_id, p));

        Ok(OnUpdateOk::Complete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, World};
    use crate::player::Player;
    use crate::vanject::Vanject;

    #[test]
    fn set_world_cleans_old_world_when_player_was_already_bound() {
        let mut srv = Server::new(Default::default());
        let mut game = Game::new(1);
        let client_id: ClientID = 11;
        game.attach_player(Player::new(client_id));

        let world1 = Rc::new(RefCell::new(World::new(1, 100)));
        game.worlds.push(Rc::clone(&world1));
        game.place_player(client_id, &world1.borrow());

        let mut vanger = Vanject::create_from_slice(&[
            1, 0, 9, 4, // id
            6, 0, 0, 0, // time
            10, 0, // x
            20, 0, // y
            15, 0, // radius
            7, 8,
        ])
        .unwrap();
        vanger.player_bind_id = 1;
        let vanger_id = vanger.id;
        game.vanjects.insert(vanger_id, vanger);

        srv.games.insert(1, game);

        let packet = Packet::new(Action::SET_WORLD, &[2, 100, 0]);
        srv.set_world(&packet, client_id).unwrap();

        let game = srv.games.get(&1).unwrap();
        let player = game.get_player(client_id).unwrap();
        assert_eq!(player.world.as_ref().unwrap().borrow().id, 2);
        assert!(!game.vanjects.contains_key(&vanger_id));
    }
}
