use crate::protocol::{Action, NetTransportSend, Packet};
use crate::vanject::{VanjectError, NID};
use crate::Server;
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
