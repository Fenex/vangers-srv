use ::tracing::{debug, warn};

use crate::client::ClientID;
use crate::protocol::{Action, NetTransportSend, Packet};
use crate::vanject::*;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum CreateObjectError {
    #[error("fail read slice as vanject: [{0}]")]
    SliceToVanjectParse(VanjectError),
    #[error("player with `client_id`={0} not found")]
    PlayerNotFound(ClientID),
    #[error("player with `client_id`={0} not bind")]
    PlayerNotBind(ClientID),
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_CreateObject {
    fn create_object(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_CreateObject for Server {
    #[tracing::instrument(skip_all)]
    fn create_object(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let mut vanject = match Vanject::create_from_slice(&packet.data) {
            Ok(vanject) => vanject,
            Err(err) => return Err(CreateObjectError::SliceToVanjectParse(err).into()),
        };

        let game = match self.get_mut_game_by_clientid(client_id) {
            Some(game) => game,
            None => return Err(CreateObjectError::PlayerNotFound(client_id).into()),
        };

        if game.vanjects.contains_key(&vanject.id) {
            debug!("VANJECT with id=`{}` already exists", vanject.id);
        } else {
            let player = game.get_mut_player(client_id).unwrap();
            if vanject.bind_to_player(player).is_err() {
                return Err(CreateObjectError::PlayerNotBind(client_id).into());
            }

            if vanject.get_type() == NID::VANGER {
                player.pos = vanject.pos;

                if player.set_body(&vanject.body).is_err() {
                    warn!("NID::VANGER: set body failed");
                } else {
                    let data = vanject.to_vangers_byte();
                    let answer = Packet::new(Action::UPDATE_OBJECT, &data);

                    let data = std::iter::empty()
                        .chain(&[vanject.player_bind_id])
                        .chain(&vanject.pos.to_vangers_byte())
                        .copied()
                        .collect::<Vec<_>>();
                    let player_position = Packet::new(Action::PLAYERS_POSITION, &data);

                    self.notify_game(client_id, &answer);
                    self.notify_game(client_id, &player_position);
                }
            } else {
                // #IF: vanject.get_type() != NID::VANGER
                let data = vanject.to_vangers_byte();
                let answer = Packet::new(Action::UPDATE_OBJECT, &data);

                if !vanject.is_players() {
                    // world->process_create;
                    self.notify_game(client_id, &answer);
                } else {
                    if vanject.is_non_global() {
                        // world->process_create_inventory()
                        debug!(
                            "Added inventory vanject {:?} to player {}",
                            &vanject.id.to_le_bytes(),
                            vanject.player_bind_id
                        );
                        self.notify_game(client_id, &answer);
                    } else {
                        // game->process_create_globals()
                        self.notify_game(client_id, &answer);
                    }
                }
            }

            self.get_mut_game_by_clientid(client_id)
                .unwrap()
                .vanjects
                .insert(vanject.id, vanject);
        }

        Ok(OnUpdateOk::Complete)
    }
}
