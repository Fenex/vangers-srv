use crate::Server;
use crate::protocol::{Action, NetTransportSend, Packet};
use crate::vanject::{NID, VanjectError};
use crate::{client::ClientID, utils::slice_le_to_i32};

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum UpdateObjectError {
    #[error("fail read slice as vanject: [too small slice]")]
    SliceTooSmall,
    #[error("fail read slice as vanject: [{0}]")]
    SliceToVanjectParse(#[from] VanjectError),
    #[error("player with `client_id`={0} not found")]
    PlayerNotFound(ClientID),
    #[error("vanject with `id`={0} not found")]
    VanjectNotFound(i32),
    #[error("player with `client_id`={0} not bind")]
    PlayerNotBind(ClientID),
    #[error("vanject with `id`={0} is not owned by player_bind_id={1}")]
    NotOwner(i32, u8),
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_UpdateObject {
    fn update_object(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_UpdateObject for Server {
    #[tracing::instrument(skip_all)]
    fn update_object(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        if packet.data.len() < 4 {
            Err(UpdateObjectError::SliceTooSmall)?
        }

        let vanject_id = slice_le_to_i32(&packet.data[0..4]);

        let game = self
            .get_mut_game_by_clientid(client_id)
            .ok_or(UpdateObjectError::PlayerNotFound(client_id))?;

        let player_bind_id = game
            .get_player(client_id)
            .expect("we got game by this player in line above")
            .bind
            .map(|bind| bind.id())
            .ok_or(UpdateObjectError::PlayerNotBind(client_id))?;

        let mut packets: Vec<Packet> = vec![];

        match game.vanjects.get_mut(&vanject_id) {
            Some(vanject) => {
                if vanject.player_bind_id != player_bind_id {
                    Err(UpdateObjectError::NotOwner(vanject_id, player_bind_id))?;
                }

                vanject
                    .update_from_slice(&packet.data)
                    .map_err(UpdateObjectError::SliceToVanjectParse)?;

                vanject.player_bind_id = player_bind_id;
                if vanject.get_type() == NID::VANGER {
                    let data = std::iter::empty()
                        .chain(&[vanject.player_bind_id])
                        .chain(&vanject.pos.to_vangers_byte())
                        .copied()
                        .collect::<Vec<_>>();
                    packets.push(Packet::new(Action::PLAYERS_POSITION, &data));
                }

                packets.push(Packet::new(
                    Action::UPDATE_OBJECT,
                    &vanject.to_vangers_byte(),
                ));
            }
            None => Err(UpdateObjectError::VanjectNotFound(vanject_id))?,
        }

        for p in packets {
            self.notify_game(client_id, &p);
        }

        Ok(OnUpdateOk::Complete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;
    use crate::player::Player;
    use crate::vanject::Vanject;

    fn make_server_with_owned_vanject() -> (Server, ClientID, ClientID, i32) {
        let mut srv = Server::new(Default::default());
        let mut game = Game::new(1);

        let owner_client: ClientID = 11;
        let other_client: ClientID = 22;
        game.attach_player(Player::new(owner_client));
        game.attach_player(Player::new(other_client));

        let mut vanject = Vanject::create_from_slice(&[
            2, 1, 1, 1, // id
            6, 0, 0, 0, // time
            10, 0, // x
            20, 0, // y
            15, 0, // radius
            1, 2, 3,
        ])
        .unwrap();
        vanject.player_bind_id = 1;
        let vanject_id = vanject.id;
        game.vanjects.insert(vanject_id, vanject);

        srv.games.insert(1, game);
        (srv, owner_client, other_client, vanject_id)
    }

    #[test]
    fn rejects_updates_from_non_owner() {
        let (mut srv, _owner_client, other_client, vanject_id) = make_server_with_owned_vanject();
        let packet = Packet::new(
            Action::UPDATE_OBJECT,
            &[
                2, 1, 1, 1, // id
                7, 0, 0, 0, // time
                11, 0, // x
                21, 0, // y
                9, 9,
            ],
        );

        let err = srv.update_object(&packet, other_client).unwrap_err();
        match err {
            OnUpdateError::UpdateObjectError(UpdateObjectError::NotOwner(id, bind_id)) => {
                assert_eq!(id, vanject_id);
                assert_eq!(bind_id, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let game = srv.games.get(&1).unwrap();
        let vanject = game.vanjects.get(&vanject_id).unwrap();
        assert_eq!(vanject.time, 6);
        assert_eq!(vanject.pos.x, 10);
        assert_eq!(vanject.pos.y, 20);
        assert_eq!(vanject.player_bind_id, 1);
    }
}
