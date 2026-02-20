use std::ffi::CStr;

use ::tracing::{error, info};

use crate::Server;
use crate::client::ClientID;
use crate::protocol::Packet;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum SetGameDataError {
    #[error("parse name failed")]
    NameParse,
    #[error("name is empty")]
    NameEmpty,
    #[error("fail read slice as game config: {0}")]
    SliceToGameConfigParse(&'static str),
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
            Some(g) if g.is_configured() => Err(SetGameDataError::AlreadyConfigured(g.id))?,
            Some(g) => g,
            None => Err(SetGameDataError::PlayerNotFound(client_id))?,
        };

        let name = match CStr::from_bytes_until_nul(&packet.data) {
            Ok(name) if name.is_empty() => Err(SetGameDataError::NameEmpty)?,
            Ok(name) => name,
            Err(err) => {
                error!("cannot extract name of the game: {err:?}");
                Err(SetGameDataError::NameParse)?
            }
        };

        game.set_config(&packet.data[name.to_bytes_with_nul().len()..])
            .inspect(|_| {
                info!(
                    "changed config for game_id=`{}`: {:?}",
                    game.id, game.config
                );
                game.name = name.to_bytes_with_nul().to_vec();
            })
            .map_err(SetGameDataError::SliceToGameConfigParse)?;

        Ok(OnUpdateOk::Complete)
    }
}
