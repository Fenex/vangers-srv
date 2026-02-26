use std::borrow::Cow;
use std::io::Write;

use tracing::warn;

use crate::client::ClientID;
use crate::protocol::Packet;
use crate::utils;
use crate::{Server, player::Player};

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum DirectSendingError {
    #[error("given data is too small")]
    Parse,
    #[error("cannot parse c-string")]
    String,
    #[error("client with client_id=`{0}` is out of all games")]
    ClientIsOutOfGames(ClientID),
    #[error("player with client_id=`{0}` is not found in game_id=`{1}`")]
    PlayerNotFound(ClientID, u32),
    #[error("player with client_id=`{0}` is not bind in game_id=`{1}`")]
    PlayerNotBind(ClientID, u32),
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
    #[tracing::instrument(skip_all)]
    fn direct_sending(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        if packet.data.len() < 4 + 1 + 1 {
            return Err(DirectSendingError::Parse.into());
        }

        let game = self
            .get_game_by_clientid(client_id)
            .ok_or(DirectSendingError::ClientIsOutOfGames(client_id))?;

        let mask = utils::slice_le_to_u32(&packet.data[0..4]);

        // storages binded player_id by client_id
        let mut player: Option<&Player> = None;
        // storages all client_ids for sending to
        let mut client_ids = vec![];

        for p in &game.players {
            if let Some(bind) = p.bind {
                if p.client_id == client_id {
                    // we find transmitter client
                    player = Some(p);
                } else if bind.mask() as u32 & mask != 0 {
                    // we find valid reciever client
                    client_ids.push(p.client_id)
                }
            }
        }

        let player = player.ok_or(DirectSendingError::PlayerNotFound(client_id, game.id))?;
        if player.bind.is_none() || player.auth.is_none() {
            Err(DirectSendingError::PlayerNotBind(client_id, game.id))?
        }
        let player_id = player
            .bind
            .expect("we check for none a few rows above")
            .id();

        let Some(msg) = utils::get_first_cstr(&packet.data[4..]) else {
            Err(DirectSendingError::String)?
        };

        let mut msg = Cow::Borrowed(msg);

        // TODO: take out to config this constant
        const LIMIT_MSG_LEN: usize = 140;
        if msg.len() > LIMIT_MSG_LEN {
            warn!(
                "direct message length is too big, max length: `{}`, given length: `{}`, the message will be cut",
                LIMIT_MSG_LEN,
                msg.len()
            );
            msg = Cow::Owned({
                let mut buffer = Vec::with_capacity(LIMIT_MSG_LEN);
                buffer.write_all(&msg[0..LIMIT_MSG_LEN - 3 - 1]).ok();
                buffer.write_all(b"...").ok();
                buffer.push(0);
                buffer
            })
        }

        let data = std::iter::empty()
            .chain(&[player_id])
            .chain(&msg[..])
            .copied()
            .collect::<Vec<_>>();

        let answer = match packet.create_answer(data) {
            Some(packet) => packet,
            None => return Err(OnUpdateError::ResponsePacketTypeNotExist(packet.action)),
        };

        self.clients
            .iter_mut()
            .filter(|c| client_ids.contains(&c.id))
            .for_each(|c| c.send(&answer));

        Ok(OnUpdateOk::Complete)
    }
}
