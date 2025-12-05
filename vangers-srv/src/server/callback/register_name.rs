use std::ffi::CStr;

use ::tracing::info;

use crate::client::ClientID;
use crate::protocol::Packet;
use crate::Server;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum RegisterNameError {
    #[error("player with `client_id`={0} not found")]
    PlayerNotFound(ClientID),
    #[error("name or password is not a C-style string")]
    NameOrPasswordParse,
    #[error("player with `client_id`={0} not bind to client object")]
    PlayerNotBind(ClientID),
}

#[allow(non_camel_case_types)]
pub(super) trait OnUpdate_RegisterName {
    fn register_name(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError>;
}

impl OnUpdate_RegisterName for Server {
    #[tracing::instrument(skip_all)]
    fn register_name(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let player = match self.games.get_mut_player_by_client_id(client_id) {
            Some(player) => player,
            None => return Err(RegisterNameError::PlayerNotFound(client_id).into()),
        };

        if player.bind.is_none() {
            Err(RegisterNameError::PlayerNotBind(client_id))?
        }

        let login = CStr::from_bytes_until_nul(&packet.data)
            .map_err(|_| RegisterNameError::NameOrPasswordParse)?;
        let pwd = CStr::from_bytes_until_nul(&packet.data[login.count_bytes() + 1..])
            .map_err(|_| RegisterNameError::NameOrPasswordParse)?;

        player.set_auth(login.to_bytes_with_nul(), pwd.to_bytes_with_nul());
        info!(
            "set name {:?} for player_id=`{}`",
            login,
            player.bind.unwrap().id()
        );

        let data = std::iter::empty()
            .chain(&player.bind.unwrap().id().to_le_bytes())
            .chain(login.to_bytes_with_nul())
            .copied()
            .collect::<Vec<_>>();

        let answer = packet.create_answer(data).unwrap();
        self.notify_game(client_id, &answer);

        Ok(OnUpdateOk::Complete)
    }
}
