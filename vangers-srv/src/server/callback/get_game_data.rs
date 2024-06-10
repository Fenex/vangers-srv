use crate::client::ClientID;
use crate::protocol::{NetTransportSend, Packet};
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum GetGameDataError {
    #[error("player with client_id `{0}` not found")]
    PlayerNotFound(ClientID),
    #[error("game with id `{0}` is not configured")]
    NotConfigured(u32),
}

impl From<GetGameDataError> for OnUpdateError {
    fn from(from: GetGameDataError) -> Self {
        Self::GetGameDataError(from)
    }
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_GetGameData {
    fn get_game_data(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_GetGameData for Server {
    fn get_game_data(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let game = match self.get_game_by_clientid(client_id) {
            Some(g) => g,
            None => return Err(GetGameDataError::PlayerNotFound(client_id).into()),
        };

        if !game.is_configured() {
            return Err(GetGameDataError::NotConfigured(game.id).into());
        }

        let data = std::iter::empty()
            .chain(&game.name)
            .chain(&game.config.as_ref().unwrap().to_vangers_byte())
            .map(|&b| b)
            .collect();

        packet
            .create_answer(data)
            .and_then(|p| Some(OnUpdateOk::Response(p)))
            .ok_or(OnUpdateError::ResponsePacketTypeNotExist(packet.action))
    }
}
