use ::log::info;

use crate::client::ClientID;
use crate::player::Player;
use crate::protocol::*;
use crate::utils::slice_le_to_i32;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum AttachToGameError {
    #[error("game with id `{0}` not found")]
    NotExists(u32),
    #[error("required byte `game_id` not found")]
    IdEmpty,
    #[error("game have no free player slots")]
    Full(u32),
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_AttachToGame {
    fn attach_to_game(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_AttachToGame for Server {
    fn attach_to_game(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        if packet.data.len() != 4 {
            return Err(AttachToGameError::IdEmpty.into());
        }

        let gmid = match slice_le_to_i32(&packet.data) {
            0 => {
                let gmid = self.get_game_uniq_id();
                self.games.create(gmid).ok();
                gmid as i32
            }
            gmid => gmid,
        };

        let game = match self.games.get_mut_game_by_id(gmid as u32) {
            Some(game) => game,
            None => return Err(AttachToGameError::NotExists(gmid as u32).into()),
        };

        let player_id = match game.attach_player(Player::new(client_id)) {
            None => return Err(AttachToGameError::Full(game.id).into()),
            Some(p_id) => p_id,
        };

        info!("=======!= ATTACHED PLAYER ID: {} =!=======", player_id);

        let data = {
            // vanject ID offsets.
            // it is need to correct sync ID of vanjects if at least one player slot will
            // be free before. I don't know what actually algorithm do, but it just works
            let offsets = {
                let mut offsets = [0u16; 16];
                for (id, v) in &game.vanjects {
                    let index = ((id >> 16) & 63) as usize;
                    if v.get_station() == player_id as i32 && offsets[index] < (id & 0xFFFF) as u16
                    {
                        offsets[index] = (id & 0xFFFF) as u16;
                    }
                }
                // it is possible to use `mem::transmute` or `byteorder` crate instead of creating new `Vec`
                offsets
                    .iter()
                    .flat_map(|&o| if o != 0 { o + 1 } else { o }.to_le_bytes().to_vec())
                    .collect::<Vec<_>>()
            };

            // Game(4)
            // Configured(1) = 1 or 0
            // GameBirthTime(4)
            // Client_ID (player_id in rust)(1)
            // object_ID_offsets[16](short)
            std::iter::empty()
                .chain(&game.id.to_le_bytes())
                .chain(&(if game.is_configured() { 1u8 } else { 0u8 }).to_le_bytes())
                .chain(&(game.birth_time.as_secs_u32() as i32).to_le_bytes())
                .chain(&player_id.to_le_bytes())
                .chain(&offsets[..]) //object_ID_offsets
                .map(|&b| b)
                .collect()
        };

        let packets = game
            .vanjects
            .iter()
            // .filter(|(_, v)| !v.is_non_global())
            .map(|(_, v)| v.to_vangers_byte())
            .map(|v| Packet::new(Action::UPDATE_OBJECT, &v[..]))
            .collect::<Vec<_>>();

        // dbg!("ATTACH_TO_GAME", player_id, &game);

        if let Some(packet) = packet.create_answer(data) {
            self.notify_player(client_id, &packet);
        }

        if self.clients.iter().find(|c| c.id == client_id && c.protocol > 1).is_some() {
            let now = std::time::SystemTime::now();
            let unix = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32;
            let packet = Packet::new(Action::Z_TIME_RESPONSE, &unix.to_le_bytes());
            self.notify_player(client_id, &packet);
        }

        packets
            .iter()
            .for_each(|packet| self.notify_player(client_id, &packet));

        Ok(OnUpdateOk::Complete)
    }
}
