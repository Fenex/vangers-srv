use ::log::warn;

use crate::client::ClientID;
use crate::protocol::{NetTransportSend, Packet};
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum TotalPlayersDataQueryError {
    #[error("player with client_id `{0}` not found")]
    PlayerNotFound(ClientID),
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_TotalPlayersDataQuery {
    fn total_players_data_query(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_TotalPlayersDataQuery for Server {
    fn total_players_data_query(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let game = match self.get_mut_game_by_clientid(client_id) {
            Some(game) => game,
            None => return Err(TotalPlayersDataQueryError::PlayerNotFound(client_id).into()),
        };

        let mut data = vec![game.players.len() as u8];
        let mut players_count = 0;
        for player in &game.players {
            let id = match player.bind {
                Some(bind) => bind.id(),
                None => continue,
            };

            let status = player.status as u8;

            let world = match player.world {
                Some(ref world) => world.borrow().id,
                None => 0u8,
            };

            let name = match player.auth {
                Some(ref auth) => auth.name(),
                None => b"[UNDEFINED]\0", // is it possible (?)
            };

            let body = match player.body {
                Some(ref body) => body.to_vangers_byte(),
                None => {
                    warn!("Player with (bind_id={} client_id={}) has no `body` property, ignored the player", id, player.client_id);
                    continue;
                }
            };

            let mut p_data = std::iter::empty()
                .chain(&[id])
                .chain(&[status])
                .chain(&[world])
                .chain(&player.pos.to_vangers_byte())
                .chain(name)
                .chain(&body)
                .copied()
                .collect::<Vec<_>>();

            data.append(&mut p_data);

            players_count += 1;
        }

        data[0] = players_count;

        packet
            .create_answer(data)
            .map(OnUpdateOk::Response)
            .ok_or(OnUpdateError::ResponsePacketTypeNotExist(packet.action))
    }
}
