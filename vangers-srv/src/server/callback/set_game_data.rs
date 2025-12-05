use crate::client::ClientID;
use crate::protocol::Packet;
use crate::utils::get_first_cstr;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum SetGameDataError {
    #[error("parse name failed")]
    NameParse,
    #[error("name is empty")]
    NameEmpty,
    #[error("fail read slice as game config")]
    SliceToGameConfigParse,
    #[error("player with client_id `{0}` not found")]
    PlayerNotFound(ClientID),
    #[error("game with game_id `{0}` already configured")]
    AlreadyConfigured(u32),
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
    #[tracing::instrument(skip_all)]
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
            Some([]) => return Err(SetGameDataError::NameEmpty.into()),
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
