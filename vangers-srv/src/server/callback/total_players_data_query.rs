use std::fmt;

use crate::client::ClientID;
use crate::protocol::{NetTransportSend, Packet};
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug)]
pub enum TotalPlayersDataQueryError {
    PlayerNotFound(ClientID),
}

impl fmt::Display for TotalPlayersDataQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerNotFound(id) => write!(f, "player with client_id `{}` not found", id),
        }
    }
}

impl From<TotalPlayersDataQueryError> for OnUpdateError {
    fn from(from: TotalPlayersDataQueryError) -> Self {
        Self::TotalPlayersDataQueryError(from)
    }
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
                    println!("ERR: Player with (bind_id={} client_id={}) has no `body` property, ignored the player", id, player.client_id);
                    continue;
                }
            };

            let mut p_data = std::iter::empty()
                .chain(&[id])
                .chain(&[status])
                .chain(&[world])
                .chain(&player.pos.to_vangers_byte())
                .chain(&name[..])
                .chain(&body)
                .map(|&b| b)
                .collect::<Vec<_>>();

            data.append(&mut p_data);

            players_count += 1;
        }

        data[0] = players_count;

        packet
            .create_answer(data)
            .and_then(|p| Some(OnUpdateOk::Response(p)))
            .ok_or(OnUpdateError::ResponsePacketTypeNotExist(packet.action))
    }
}
