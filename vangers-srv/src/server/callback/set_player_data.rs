use std::fmt;

use crate::client::ClientID;
use crate::game::Type as GameType;
use crate::protocol::Packet;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug)]
pub enum SetPlayerDataError {
    PlayerNotFound(ClientID),
    PlayerNotBind(ClientID),
    SliceToBodyParse(GameType),
}

impl fmt::Display for SetPlayerDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerNotFound(id) => write!(f, "player with client_id `{}` not found", id),
            Self::PlayerNotBind(id) => write!(f, "player with client_id `{}` not bind", id),
            Self::SliceToBodyParse(gmtype) => {
                write!(f, "fail read slice as body (gmtype is `{:?}`)", gmtype)
            }
        }
    }
}

impl From<SetPlayerDataError> for OnUpdateError {
    fn from(from: SetPlayerDataError) -> Self {
        Self::SetPlayerDataError(from)
    }
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_SetPlayerData {
    fn set_player_data(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_SetPlayerData for Server {
    fn set_player_data(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let game = match self.get_mut_game_by_clientid(client_id) {
            Some(game) => game,
            None => return Err(SetPlayerDataError::PlayerNotFound(client_id).into()),
        };

        let gmtype = game.get_gmtype();

        let player_id = {
            let player = game
                .players
                .iter_mut()
                .find(|p| p.client_id == client_id)
                .unwrap();
            match (player.set_body(&packet.data), player.bind) {
                (Ok(_), Some(bind)) => bind.id(),
                (Ok(_), None) => return Err(SetPlayerDataError::PlayerNotBind(client_id).into()),
                (Err(_), _) => return Err(SetPlayerDataError::SliceToBodyParse(gmtype).into()),
            }
        };

        //TODO: dispatch event `PLAYERS_RATING` which depends on game.GameType

        let data = std::iter::empty()
            .chain(&[player_id])
            .chain(&packet.data)
            .map(|&b| b)
            .collect::<Vec<_>>();

        let packet = match packet.create_answer(data) {
            Some(answer) => answer,
            None => return Err(OnUpdateError::ResponsePacketTypeNotExist(packet.action)),
        };

        // dbg!("SET_PLAYER_DATA", player_id, &game);

        self.notify_game(client_id, &packet);

        Ok(OnUpdateOk::Complete)
    }
}
