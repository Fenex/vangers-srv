use std::convert::TryFrom;

use ::enum_primitive_derive::Primitive;
use ::num_traits::{FromPrimitive, ToPrimitive};

// Event's flags
// pub const AUXILIARY_EVENT: u8 = 0x80;
// pub const ECHO_EVENT: u8 = 0x20;

pub trait NetTransportSend {
    fn to_vangers_byte(&self) -> Vec<u8>;
}

pub trait NetTransportReceive: Sized {
    fn from_slice(slice: &[u8]) -> Option<Self>;
}

pub trait NetTransport: NetTransportSend + NetTransportReceive {}
impl<T> NetTransport for T where T: NetTransportSend + NetTransportReceive {}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Primitive, Eq, PartialEq, Debug)]
pub enum Action {
    UNKNOWN = 0x00,

    CREATE_OBJECT = 0x02,
    DELETE_OBJECT = 0x04,
    UPDATE_OBJECT = 0x08,
    HIDE_OBJECT = 0x0C,

    // request to server
    GAMES_LIST_QUERY = 0x81,
    TOP_LIST_QUERY = 0x82,
    ATTACH_TO_GAME = 0x83,
    RESTORE_CONNECTION = 0x84,
    CLOSE_SOCKET = 0x86,
    REGISTER_NAME = 0x88,
    SERVER_TIME_QUERY = 0x89,
    SET_WORLD = 0x8B,
    LEAVE_WORLD = 0x8C,
    SET_POSITION = 0x8D,
    TOTAL_PLAYERS_DATA_QUERY = 0x91,
    SET_GAME_DATA = 0x92,
    GET_GAME_DATA = 0x93,
    SET_PLAYER_DATA = 0x94,
    DIRECT_SENDING = 0x95,

    // response to client
    GAMES_LIST_RESPONSE = 0xC1,
    TOP_LIST_RESPONSE = 0xC2,
    TOTAL_LIST_OF_PLAYERS_DATA = 0xCC,
    ATTACH_TO_GAME_RESPONSE = 0xC3,
    RESTORE_CONNECTION_RESPONSE = 0xC4,
    SERVER_TIME = 0xC6,
    SERVER_TIME_RESPONSE = 0xC7,
    SET_WORLD_RESPONSE = 0xC8,
    GAME_DATA_RESPONSE = 0xCD,
    DIRECT_RECEIVING = 0xCE,
    PLAYERS_NAME = 0xD5,
    PLAYERS_POSITION = 0xCF,
    PLAYERS_WORLD = 0xD1,
    PLAYERS_STATUS = 0xD2,
    PLAYERS_DATA = 0xD3,
    PLAYERS_RATING = 0xD4,
    Z_TIME_RESPONSE = 0xE3
}

impl Action {
    /// Converts `action` from request-type to response-type if possible.
    pub fn request_to_response(&self) -> Option<Self> {
        let action_response = match self {
            Action::GAMES_LIST_QUERY => Action::GAMES_LIST_RESPONSE,
            Action::ATTACH_TO_GAME => Action::ATTACH_TO_GAME_RESPONSE,
            Action::SERVER_TIME_QUERY => Action::SERVER_TIME,
            Action::TOTAL_PLAYERS_DATA_QUERY => Action::TOTAL_LIST_OF_PLAYERS_DATA,
            Action::REGISTER_NAME => Action::PLAYERS_NAME,
            Action::SET_PLAYER_DATA => Action::PLAYERS_DATA,
            Action::SET_GAME_DATA => return None, // no answer
            Action::GET_GAME_DATA => Action::GAME_DATA_RESPONSE,
            Action::SET_WORLD => Action::SET_WORLD_RESPONSE,
            Action::DIRECT_SENDING => Action::DIRECT_RECEIVING,
            _ => return None,
        };

        Some(action_response)
    }
}

#[derive(Clone, Debug)]
/// Represents packet of event (communication data between server and clients).
pub struct Packet {
    /// Size of the packet.
    ///
    /// If Packet was constructed via call `from_slice` method,
    /// the field will storage first two bytes of income slice
    /// even if the length of an income slice is different.
    ///
    /// In other cases event_size must be  equal `1 + data.len()`
    /// where `1` is size of `action` field (type `u8`).
    pub event_size: i16,

    /// Represents packet type.
    pub action: Action,
    /// Represents real packet type as byte
    pub real_action: u8,
    /// Storages all packet bytes starting from 4th byte.
    pub data: Vec<u8>,
}

impl Packet {
    /// Returns `Packet` as array of bytes that ready to sending outside.
    pub fn as_bytes(&self) -> Vec<u8> {
        let action: u8 = if self.action == Action::UNKNOWN {
            self.real_action
        } else {
            self.action.to_u8().unwrap()
        };

        let event_size = 1 + &self.data.len();
        let event_size = i16::try_from(event_size).unwrap();
        std::iter::empty()
            .chain(&event_size.to_le_bytes())
            .chain(&[action])
            .chain(&self.data)
            .map(|&u| u)
            .collect()
    }

    /// Returns new `Packet` with corrected answer-action type based on
    /// action type of `self`.
    /// The packet will be include given `data` and correct calculated `event_size`.
    ///
    /// `None` will be returned if an action type of `self` can not be
    /// converted to answer-action.
    pub fn create_answer(&self, data: Vec<u8>) -> Option<Self> {
        self.action
            .request_to_response()
            .and_then(|action| {
                let event_size = i16::try_from(data.len() + 1).ok()?;
                Some((action, event_size))
            })
            .and_then(|(action, event_size)| {
                Some(Self {
                    action,
                    real_action: action as u8,
                    data,
                    event_size,
                })
            })
    }

    pub fn new(action: Action, data: &[u8]) -> Self {
        let event_size = i16::try_from(data.len() + 1).unwrap();
        Self {
            action,
            real_action: action as u8,
            data: data.to_owned(),
            event_size,
        }
    }

    pub fn from_slice(buff: &[u8]) -> Self {
        if buff.len() < 3 {
            Self::default()
        } else {
            let mut event_size_slice = [0, 0];
            event_size_slice.clone_from_slice(&buff[0..2]);

            let event_size = i16::from_le_bytes(event_size_slice);
            let real_action = buff[2];
            let action = Action::from_u8(buff[2]).unwrap_or(Action::UNKNOWN);
            let data = buff[3..].to_vec();

            if data.len() + 1 != usize::try_from(event_size).unwrap() {
                println!(
                    "Warning: `event_size` ({}) and real length ({}) of the packet are differents!",
                    event_size,
                    data.len() + 1
                );
            }

            // Save `event_size` from income slice here only.
            // In other places, event_size must be calculated based on length of the `data`.
            Self {
                event_size,
                action,
                real_action,
                data,
            }
        }
    }
}

impl Default for Packet {
    fn default() -> Self {
        Self {
            event_size: 1,
            real_action: 0x00,
            action: Action::UNKNOWN,
            data: vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// The bytecode that doesn't represents any correct `Action` code.
    const UNDEFINED_ACTION: u8 = 0x01;

    #[test]
    fn check_for_undefined_action() {
        // This test will fail if a new action with bytecode `UNDEFINED_ACTION`
        // will be added in the future. To fix that you will need to
        // change `UNDEFINED_ACTION` constant to something else.
        // Actually, the line tests for correct running tests below :)
        assert!(Action::from_u8(UNDEFINED_ACTION).is_none());
    }

    const DATA_UNDEFINED: &'static [u8] = &[0x04, 0x00, UNDEFINED_ACTION, 0xAA, 0xAA, 0xAA];

    /* behavior has been changed by declare `real_action` field, no need below code anymore (?): */
    // /// If the packet has an undefined action,
    // /// then byte `Action` will be replaced by Action::UNDEFINED (0x00)
    // const DATA_UNDEF_FIX: &'static [u8] =
    //     &[0x04, 0x00, /* REPLACE BYTE */ 0x00, 0xAA, 0xAA, 0xAA];

    const DATA_UNKNOWN_3: &'static [u8] = &[0x04, 0x00, 0x00, 0x11, 0x11, 0x11];
    const DATA_GLQ_1: &'static [u8] = &[0x01, 0x00, 0x81, 0xD1, 0xD1, 0xD1];
    const DATA_GLQ_1_FIXED_SIZE_4: &'static [u8] = &[0x04, 0x00, 0x81, 0xD1, 0xD1, 0xD1];
    const DATA_GLQ_4: &'static [u8] = &[0x04, 0x00, 0x81, 0xD4, 0xD4, 0xD4];

    #[test]
    fn test_packet_from_slice() {
        let p = Packet::from_slice(DATA_GLQ_1);
        assert_eq!(1i16, p.event_size);
        assert_eq!(&Action::GAMES_LIST_QUERY, &p.action);
        assert_eq!(&DATA_GLQ_1_FIXED_SIZE_4[3..6], &p.data[..]);
        assert_eq!(DATA_GLQ_1_FIXED_SIZE_4, &p.as_bytes()[..]);

        let p = Packet::from_slice(&DATA_GLQ_4);
        assert_eq!(4i16, p.event_size);
        assert_eq!(&Action::GAMES_LIST_QUERY, &p.action);
        assert_eq!(&DATA_GLQ_4[3..6], &p.data[..]);
        assert_eq!(DATA_GLQ_4, &p.as_bytes()[..]);

        // Action is undefined
        let p = Packet::from_slice(&DATA_UNDEFINED);
        assert_eq!(4i16, p.event_size);
        assert_eq!(&Action::UNKNOWN, &p.action);
        assert_eq!(&DATA_UNDEFINED[3..6], &p.data[..]);
        /* `real_action` changed the assertion:
        // assert_eq!(DATA_UNDEF_FIX, &p.as_bytes()[..]);
         */
        assert_eq!(DATA_UNDEFINED, &p.as_bytes()[..]);

        // Action equals 0x00 (UNKNOWN)
        let p = Packet::from_slice(&DATA_UNKNOWN_3);
        assert_eq!(4i16, p.event_size);
        assert_eq!(&Action::UNKNOWN, &p.action);
        assert_eq!(&DATA_UNKNOWN_3[3..6], &p.data[..]);
        assert_eq!(DATA_UNKNOWN_3, &p.as_bytes()[..]);

        // empty `data`
        let p = Packet::from_slice(&DATA_UNKNOWN_3[0..3]);
        assert_eq!(4i16, p.event_size);
        assert_eq!(&Action::UNKNOWN, &p.action);
        assert_eq!(&[] as &[u8], &p.data[..]);
        assert_eq!(&[0x01, 0x00, 0x00], &p.as_bytes()[..]);
    }

    #[test]
    fn test_packet_new() {
        let p = Packet::new(Action::SERVER_TIME, &[]);
        assert_eq!(1i16, p.event_size);
        assert_eq!(&Action::SERVER_TIME, &p.action);
        assert_eq!(&[] as &[u8], &p.data[..]);
        assert_eq!(&[0x01, 0x00, 0xC6], &p.as_bytes()[..]);

        let p = Packet::new(Action::SERVER_TIME, &[0xEE, 0xEE]);
        assert_eq!(3i16, p.event_size);
        assert_eq!(&Action::SERVER_TIME, &p.action);
        assert_eq!(&[0xEE, 0xEE][..], &p.data[..]);
        assert_eq!(&[0x03, 0x00, 0xC6, 0xEE, 0xEE], &p.as_bytes()[..]);
    }
}
