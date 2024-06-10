use crate::client::ClientID;
use crate::protocol::Packet;
use crate::utils::get_first_cstr;
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

impl From<RegisterNameError> for OnUpdateError {
    fn from(from: RegisterNameError) -> Self {
        Self::RegisterNameError(from)
    }
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
    fn register_name(
        &mut self,
        packet: &Packet,
        client_id: ClientID,
    ) -> Result<OnUpdateOk, OnUpdateError> {
        let player = match self.games.get_mut_player_by_client_id(client_id) {
            Some(player) => player,
            None => return Err(RegisterNameError::PlayerNotFound(client_id).into()),
        };

        let auth = get_first_cstr(&packet.data).and_then(|name| {
            // TODO: fix fail if password is empty
            if name.len() > 0 {
                if let Some(pwd) = get_first_cstr(&packet.data[name.len()..]) {
                    // if pwd.len() > 0 {
                    return Some((name, pwd));
                // }
                } else {
                    return Some((name, &[0x80, 0x80, 0x00]));
                }
            }
            None
        });

        match auth {
            Some((name, pwd)) => player.set_auth(name, pwd),
            None => return Err(RegisterNameError::NameOrPasswordParse.into()),
        }

        if player.bind.is_none() {
            return Err(RegisterNameError::PlayerNotBind(client_id).into());
        }

        assert!(auth.unwrap().0.len() != 0);

        let data = std::iter::empty()
            .chain(&player.bind.unwrap().id().to_le_bytes())
            .chain(auth.unwrap().0)
            .map(|&b| b)
            .collect::<Vec<_>>();

        let answer = packet.create_answer(data).unwrap();
        self.notify_game(client_id, &answer);

        Ok(OnUpdateOk::Complete)
    }
}
