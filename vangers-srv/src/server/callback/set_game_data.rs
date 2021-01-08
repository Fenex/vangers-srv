use std::fmt;

use crate::client::ClientID;
use crate::protocol::Packet;
use crate::utils::get_first_cstr;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug)]
pub enum SetGameDataError {
    NameParse,
    NameEmpty,
    SliceToGameConfigParse,
    PlayerNotFound(ClientID),
    AlreadyConfigured(u32),
}

impl fmt::Display for SetGameDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NameParse => write!(f, "parse name failed"),
            Self::NameEmpty => write!(f, "name is empty"),
            Self::SliceToGameConfigParse => write!(f, "fail read slice as game config"),
            Self::PlayerNotFound(p) => write!(f, "player with client_id `{}` not found", p),
            Self::AlreadyConfigured(g) => write!(f, "game with game_id `{}` already configured", g),
        }
    }
}

impl From<SetGameDataError> for OnUpdateError {
    fn from(from: SetGameDataError) -> Self {
        Self::SetGameDataError(from)
    }
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_SetGameData {
    fn set_game_data(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_SetGameData for Server {
    fn set_game_data(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let game = match self.games.get_mut_game_by_client_id(client_id) {
            Some(g) if g.is_configured() => {
                return Err(SetGameDataError::AlreadyConfigured(g.id).into())
            }
            Some(g) => g,
            None => return Err(SetGameDataError::PlayerNotFound(client_id).into()),
        };

        let name = match get_first_cstr(&packet.data) {
            Some(name) if name.len() == 0 => return Err(SetGameDataError::NameEmpty.into()),
            Some(name) => name,
            None => return Err(SetGameDataError::NameParse.into()),
        };

        match game.set_config(&packet.data[name.len()..]) {
            Ok(()) => {
                game.name = name.to_vec();
                // Ok(OnUpdateOk::Complete);
            }
            Err(_) => return Err(SetGameDataError::SliceToGameConfigParse.into()),
        }

        // dbg!("SET_GAME_DATA", &game);

        Ok(OnUpdateOk::Complete)
    }
}
