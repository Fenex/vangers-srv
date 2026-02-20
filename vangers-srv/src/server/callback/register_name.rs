use std::{
    borrow::Cow,
    ffi::{CStr, CString},
};

use ::tracing::info;

use crate::Server;
use crate::client::ClientID;
use crate::protocol::Packet;

use super::{OnUpdateError, OnUpdateOk};

#[derive(Debug, ::thiserror::Error)]
pub enum RegisterNameError {
    #[error("player with `client_id`={0} not found")]
    PlayerNotFound(ClientID),
    #[error("name or password is not a C-style string")]
    NameOrPasswordParse,
    #[error("player with `client_id`={0} not bind to client object")]
    PlayerNotBind(ClientID),
    #[error("request to set empty name")]
    NameIsNull,
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
        let player = self
            .games
            .get_mut_player_by_client_id(client_id)
            .ok_or(RegisterNameError::PlayerNotFound(client_id))?;

        let player_bind_id = player
            .bind
            .map(|bind| bind.id())
            .ok_or(RegisterNameError::PlayerNotBind(client_id))?;

        let (login, pwd) = extract_auth_data(&packet.data)?;

        player.set_auth(login.to_bytes_with_nul(), pwd.to_bytes_with_nul());
        info!("set name {:?} for player_id=`{}`", login, player_bind_id);

        let data = std::iter::empty()
            .chain(&player_bind_id.to_le_bytes())
            .chain(login.to_bytes_with_nul())
            .copied()
            .collect::<Vec<_>>();

        let answer = packet.create_answer(data).unwrap();
        self.notify_game(client_id, &answer);

        Ok(OnUpdateOk::Complete)
    }
}

fn extract_auth_data<'a>(data: &'a [u8]) -> Result<(Cow<'a, CStr>, &'a CStr), RegisterNameError> {
    let mut name = Cow::Borrowed(
        CStr::from_bytes_until_nul(&data).map_err(|_| RegisterNameError::NameOrPasswordParse)?,
    );

    if name.is_empty() {
        Err(RegisterNameError::NameIsNull)?
    }

    if data.len() <= name.to_bytes_with_nul().len() {
        Err(RegisterNameError::NameOrPasswordParse)?
    }

    let pwd = CStr::from_bytes_until_nul(&data[name.to_bytes_with_nul().len()..])
        .map_err(|_| RegisterNameError::NameOrPasswordParse)?;

    {
        // TODO: create auth service and checks for correct pwd here
    }

    if name.count_bytes() > 16 {
        name = CString::from_vec_with_nul(
            name.to_bytes_with_nul()
                .into_iter()
                .take(15)
                .chain(&[0])
                .copied()
                .collect::<Vec<_>>(),
        )
        .expect("the `name` contains *exactly* one null-terminator that we appends manually")
        .into();
    }

    Ok((name, pwd))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct() {
        assert_eq!(
            (c"auth".into(), c"pwd".into()),
            extract_auth_data(b"auth\0pwd\0\0").unwrap()
        );

        assert_eq!(
            (c"auth".into(), c"pwd".into()),
            extract_auth_data(b"auth\0pwd\0").unwrap()
        );
    }

    #[test]
    fn correct_with_login_is_too_long() {
        assert_eq!(
            (c"123456789_123456".into(), c"pwd".into()),
            extract_auth_data(b"123456789_123456\0pwd\0").unwrap(),
            "correct login with max symbols (16)"
        );

        assert_eq!(
            (c"123456789_12345".into(), c"pwd".into()),
            extract_auth_data(b"123456789_1234567\0pwd\0\0").unwrap(),
            "login greater than 16 symbols, so we shrink it to len=15"
        );

        assert_eq!(
            (c"123456789_12345".into(), c"pwd".into()),
            extract_auth_data(b"123456789_123456789_123\0pwd\0\0").unwrap(),
            "login greater than 16 symbols, so we shrink it to len=15"
        );

        assert_eq!(
            (c"123456789_12345".into(), c"".into()),
            extract_auth_data(b"123456789_123456789_123\0\0pwd\0\0").unwrap(),
            "login greater than 16 symbols, so we shrink it to len=15, pwd is empty"
        );
    }

    #[test]
    fn empty_input() {
        assert!(extract_auth_data(b"").is_err(), "missed null-terminator");
        assert!(
            extract_auth_data(b"\0").is_err(),
            "empty string (single null-terminator)"
        );
        assert!(
            extract_auth_data(b"\0rnd").is_err(),
            "empty string (single null-terminator)"
        );
        assert!(
            extract_auth_data(b"\0rnd\0").is_err(),
            "empty string (single null-terminator)"
        );
        assert!(
            extract_auth_data(b"\0\0").is_err(),
            "empty string (double null-terminator)"
        );
    }

    #[test]
    fn empty_password() {
        assert_eq!(
            (c"auth".into(), c"".into()),
            extract_auth_data(b"auth\0\0").unwrap(),
            "pwd empty is allowed until authicate service will be created"
        );

        assert_eq!(
            (c"auth".into(), c"".into()),
            extract_auth_data(b"auth\0\0asdf").unwrap(),
            "pwd empty is allowed until authicate service will be created"
        );

        assert_eq!(
            (c"auth".into(), c"".into()),
            extract_auth_data(b"auth\0\0asdf\0").unwrap(),
            "pwd empty is allowed until authicate service will be created"
        );

        assert_eq!(
            (c"auth".into(), c"".into()),
            extract_auth_data(b"auth\0\0\0").unwrap(),
            "pwd empty is allowed until authicate service will be created"
        );
    }

    #[test]
    fn invalid() {
        assert!(
            extract_auth_data(b"auth").is_err(),
            "missed null-terminator"
        );
        assert!(
            extract_auth_data(b"auth\0").is_err(),
            "missed pwd (single null-terminator)"
        );
    }
}
