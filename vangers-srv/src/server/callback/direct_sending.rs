use crate::client::ClientID;
use crate::protocol::Packet;
use crate::utils;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum DirectSendingError {
    #[error("given data is too small")]
    Parse,
    #[error("cannot parse c-string")]
    String,
    #[error("player with client_id=`{0}` not bind to connection or not exists")]
    TxPlayerNotFound(ClientID),
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_DirectSending {
    fn direct_sending(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_DirectSending for Server {
    fn direct_sending(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        if packet.data.len() < 4 + 1 + 1 {
            return Err(DirectSendingError::Parse.into());
        }

        let mask = utils::slice_le_to_u32(&packet.data[0..4]);

        // storages binded player_id by client_id
        let mut player_id: Option<u8> = None;
        // storages all client_ids for sending to
        let mut client_ids = vec![];
        if let Some(game) = self.get_game_by_clientid(client_id) {
            for p in &game.players {
                if let Some(bind) = p.bind {
                    if p.client_id == client_id {
                        // we find transmitter client
                        player_id = Some(bind.id());
                    } else if bind.mask() as u32 & mask != 0 {
                        // we find valid reciever client
                        client_ids.push(p.client_id)
                    }
                }
            }
        }

        let msg = match utils::get_first_cstr(&packet.data[4..]) {
            Some(msg) => msg,
            None => return Err(DirectSendingError::String.into()),
        };
        let player_id = match player_id {
            Some(p_id) => p_id,
            None => return Err(DirectSendingError::TxPlayerNotFound(client_id).into()),
        };
        let data = std::iter::empty()
            .chain(&[player_id])
            .chain(&msg[..])
            .map(|&b| b)
            .collect::<Vec<_>>();

        let answer = match packet.create_answer(data) {
            Some(packet) => packet,
            None => return Err(OnUpdateError::ResponsePacketTypeNotExist(packet.action)),
        };

        self.clients
            .iter_mut()
            .filter(|c| client_ids.iter().find(|&&id| id == c.id).is_some())
            .for_each(|c| c.send(&answer));

        Ok(OnUpdateOk::Complete)
    }
}
