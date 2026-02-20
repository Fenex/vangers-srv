// [Van]gers Ob[ject] structure

use crate::player::Player;
use crate::protocol::{NetTransportReceive, NetTransportSend};
use crate::utils::{slice_le_to_i16, slice_le_to_i32};
use std::fmt;

#[allow(dead_code)]
#[allow(non_snake_case)]
pub mod NID {
    const MAX_NID_VANJECT: i32 = 15;

    pub const GLOBAL: i32 = 0 << 16;

    pub const DEVICE: i32 = 1 << 16;
    pub const SLOT: i32 = 2 << 16;
    pub const SHELL: i32 = 3 << 16;

    pub const VANGER: i32 = 9 << 16;
    pub const STUFF: i32 = 11 << 16;
    pub const SENSOR: i32 = (12 << 16) | (1 << 31);
    pub const TNT: i32 = (14 << 16) | (1 << 31);
    pub const TERRAIN: i32 = (15 << 16) | (1 << 31);
}

#[allow(dead_code)]
#[inline(always)]
fn client_id(id: i32) -> i32 {
    (id >> 26) & 31
}

#[allow(dead_code)]
#[inline(always)]
pub fn get_station(id: i32) -> i32 {
    (id >> 26) & 31
}

#[allow(dead_code)]
#[inline(always)]
pub fn get_world(id: i32) -> i32 {
    (id >> 22) & 15
}

#[inline(always)]
pub fn get_vanject_type(id: i32) -> i32 {
    id & (63 << 16)
}

#[allow(dead_code)]
#[inline(always)]
pub fn is_non_static(id: i32) -> bool {
    (id as u32 & (1 << 31)) == 0u32
}

#[allow(dead_code)]
#[inline(always)]
pub fn is_players_vanject(id: i32) -> bool {
    (id as u32 & (7 << 16 + 3)) == 0u32
}

#[allow(dead_code)]
#[inline(always)]
pub fn is_private_vanject(id: i32) -> bool {
    ((id >> 16) & 63) >= 8 && ((id >> 16) & 63) <= 10
}

#[inline(always)]
pub fn is_non_global_vanject(id: i32) -> bool {
    ((id as u32 >> 16) & 63) != 0u32
}

#[allow(dead_code, non_camel_case_types)]
pub enum PlayerStatus {
    INITIAL_STATUS = 0,
    GAMING_STATUS = 1,
    FINISHED_STATUS = 2,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Pos<T> {
    pub x: T,
    pub y: T,
}

impl<T> fmt::Display for Pos<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pos ({}, {})", self.x, self.y)
    }
}

impl NetTransportReceive for Pos<i16> {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 4 {
            // panic!("slice length must be equal to 4");
            return None;
        }

        Some(Pos {
            x: slice_le_to_i16(&slice[0..2]),
            y: slice_le_to_i16(&slice[2..4]),
        })
    }
}

impl NetTransportSend for Pos<i16> {
    fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.x.to_le_bytes())
            .chain(&self.y.to_le_bytes())
            .copied()
            .collect()
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum VanjectError {
    SliceTooSmall,
    Update(VanjectUpdateError),
    Create(VanjectCreateError),
}

impl fmt::Display for VanjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SliceTooSmall => write!(f, "Length of the given slice is too small"),
            Self::Update(e) => write!(f, "Update vanject error: {}", e),
            Self::Create(e) => write!(f, "Create vanject error: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum VanjectCreateError {}

impl fmt::Display for VanjectCreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VanjectCreateERROR")
        // match self {
        //     Self::MissmatchId(expected, given) => write!(
        //         f,
        //         "missmatch vanject id: expected: {}, given: {}",
        //         expected, given
        //     ),
        // }
    }
}

impl From<VanjectCreateError> for VanjectError {
    fn from(from: VanjectCreateError) -> Self {
        Self::Create(from)
    }
}

#[derive(Debug)]
pub enum VanjectUpdateError {
    MissmatchId(i32, i32),
}

impl fmt::Display for VanjectUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissmatchId(expected, given) => write!(
                f,
                "missmatch vanject id: expected: {}, given: {}",
                expected, given
            ),
        }
    }
}

impl From<VanjectUpdateError> for VanjectError {
    fn from(from: VanjectUpdateError) -> Self {
        Self::Update(from)
    }
}

#[derive(Debug)]
pub struct Vanject {
    pub id: i32,
    pub player_bind_id: u8,
    pub time: i32,
    pub pos: Pos<i16>,
    #[allow(dead_code)]
    pub radius: i16,
    pub body: Vec<u8>,
}

impl Vanject {
    pub fn create_from_slice(slice: &[u8]) -> Result<Vanject, VanjectError> {
        if slice.len() < 14 {
            return Err(VanjectError::SliceTooSmall);
        }

        let id = slice_le_to_i32(&slice[0..4]);
        let time: i32 = slice_le_to_i32(&slice[4..8]);
        let pos = Pos::from_slice(&slice[8..12]).unwrap();
        let radius = slice_le_to_i16(&slice[12..14]);

        let body = if get_vanject_type(id) == NID::VANGER {
            if slice.len() < 15 {
                return Err(VanjectError::SliceTooSmall);
            }
            // let y_half_size_of_screen = slice[14] << 1;
            &slice[15..]
        } else {
            &slice[14..]
        };

        Ok(Vanject {
            id,
            player_bind_id: 0,
            time,
            pos,
            radius,
            body: body.to_vec(),
        })
    }

    pub fn update_from_slice(&mut self, slice: &[u8]) -> Result<(), VanjectError> {
        if slice.len() < 12 {
            return Err(VanjectError::SliceTooSmall);
        }

        let id = slice_le_to_i32(&slice[0..4]);
        if id != self.id {
            return Err(VanjectUpdateError::MissmatchId(self.id, id).into());
        }

        let time: i32 = slice_le_to_i32(&slice[4..8]);
        let pos = Pos::from_slice(&slice[8..12]).unwrap();

        let body = if get_vanject_type(id) == NID::VANGER {
            if slice.len() < 13 {
                return Err(VanjectError::SliceTooSmall);
            }
            // let y_half_size_of_screen = slice[14] << 1;
            &slice[13..]
        } else {
            &slice[12..]
        };

        self.time = time;
        self.pos = pos;
        self.body = body.to_vec();

        Ok(())
    }

    pub fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.id.to_le_bytes())
            .chain(&[self.player_bind_id])
            .chain(&self.time.to_le_bytes())
            .chain(&self.pos.x.to_le_bytes())
            .chain(&self.pos.y.to_le_bytes())
            .chain(&self.body[..])
            .copied()
            .collect::<Vec<_>>()
    }

    #[allow(dead_code)]
    pub fn bind_to_player(&mut self, p: &Player) -> Result<(), &'static str> {
        if let Some(bind) = p.bind {
            self.player_bind_id = bind.id();
            Ok(())
        } else {
            Err("player not bind")
        }
    }

    #[inline(always)]
    pub fn get_type(&self) -> i32 {
        get_vanject_type(self.id)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn get_station(&self) -> i32 {
        get_station(self.id)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn is_players(&self) -> bool {
        is_players_vanject(self.id)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn is_private(&self) -> bool {
        is_private_vanject(self.id)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn is_non_global(&self) -> bool {
        is_non_global_vanject(self.id)
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn get_world(&self) -> i32 {
        get_world(self.id)
    }
}

#[cfg(test)]
mod test {
    pub use super::*;
    use crate::utils::slice_le_to_i32;

    #[test]
    fn inline_functions_entering_escape() {
        /// Clients send below Vanjects' ID right after creating new MP game (Van-War).
        const A: [u8; 4] = [83, 0, 0, 0];
        const B: [u8; 4] = [83, 0, 64, 0];
        const C: [u8; 4] = [83, 0, 128, 0];

        let a = slice_le_to_i32(&A);
        let b = slice_le_to_i32(&B);
        let c = slice_le_to_i32(&C);

        assert_eq!(0, client_id(a));
        assert_eq!(0, client_id(b));
        assert_eq!(0, client_id(c));

        assert_eq!(0, get_station(a));
        assert_eq!(0, get_station(b));
        assert_eq!(0, get_station(c));

        assert_eq!(0, get_world(a));
        assert_eq!(1, get_world(b));
        assert_eq!(2, get_world(c));

        assert_eq!(0, get_vanject_type(a));
        assert_eq!(0, get_vanject_type(b));
        assert_eq!(0, get_vanject_type(c));

        assert_eq!(true, is_non_static(a));
        assert_eq!(true, is_non_static(b));
        assert_eq!(true, is_non_static(c));

        assert_eq!(true, is_players_vanject(a));
        assert_eq!(true, is_players_vanject(b));
        assert_eq!(true, is_players_vanject(c));

        assert_eq!(false, is_private_vanject(a));
        assert_eq!(false, is_private_vanject(b));
        assert_eq!(false, is_private_vanject(c));

        assert_eq!(false, is_non_global_vanject(a));
        assert_eq!(false, is_non_global_vanject(b));
        assert_eq!(false, is_non_global_vanject(c));
    }

    #[test]
    fn inline_functions_entering_world() {
        const A: [u8; 4] = [1, 0, 14, 132];
        let a = slice_le_to_i32(&A);

        assert_eq!(1, client_id(a));
        assert_eq!(1, get_station(a));
        assert_eq!(0, get_world(a));
        assert_eq!(917504, get_vanject_type(a));
        assert_eq!(false, is_non_static(a));
        assert_eq!(false, is_players_vanject(a));
        assert_eq!(false, is_private_vanject(a));
        assert_eq!(true, is_non_global_vanject(a));

        const B: [u8; 4] = [1, 0, 2, 4];
        let a = slice_le_to_i32(&B);
        assert_eq!(1, client_id(a));
        assert_eq!(1, get_station(a));
        assert_eq!(0, get_world(a));
        assert_eq!(131072, get_vanject_type(a));
        assert_eq!(true, is_non_static(a));
        assert_eq!(true, is_players_vanject(a));
        assert_eq!(false, is_private_vanject(a));
        assert_eq!(true, is_non_global_vanject(a));

        const C: [u8; 4] = [1, 0, 9, 4];
        let a = slice_le_to_i32(&C);
        assert_eq!(1, client_id(a));
        assert_eq!(1, get_station(a));
        assert_eq!(0, get_world(a));
        assert_eq!(589824, get_vanject_type(a));
        assert_eq!(true, is_non_static(a));
        assert_eq!(false, is_players_vanject(a));
        assert_eq!(true, is_private_vanject(a));
        assert_eq!(true, is_non_global_vanject(a));

        const D: [u8; 4] = [1, 0, 66, 4];
        let a = slice_le_to_i32(&D);
        assert_eq!(1, client_id(a));
        assert_eq!(1, get_station(a));
        assert_eq!(1, get_world(a));
        assert_eq!(131072, get_vanject_type(a));
        assert_eq!(true, is_non_static(a));
        assert_eq!(true, is_players_vanject(a));
        assert_eq!(false, is_private_vanject(a));
        assert_eq!(true, is_non_global_vanject(a));
    }

    mod create_from_slice {
        use super::*;

        #[test]
        #[allow(non_snake_case)]
        fn correct__NID_VANGER() {
            let slice = std::iter::empty()
                .chain(&[1, 0, 9, 4]) // id = 67698689
                .chain(&[6, 0, 0, 0]) // time = 6
                .chain(&[10, 0]) // pos.x = 10
                .chain(&[20, 0]) // pos.y = 20
                .chain(&[15, 0]) // radius = 15
                .chain(&[8u8]) // y_half_size_of_screen (inside NID::VANGER vajects only)
                .chain(&[1u8, 2, 3, 4, 5, 6])
                .map(|&b| b)
                .collect::<Vec<_>>();

            let v = Vanject::create_from_slice(&slice);
            assert!(v.is_ok());
            let mut v = v.unwrap();

            assert_eq!(67698689, v.id);
            assert_eq!(6, v.time);
            assert_eq!(10, v.pos.x);
            assert_eq!(20, v.pos.y);
            assert_eq!(15, v.radius);
            assert_eq!(&[1u8, 2, 3, 4, 5, 6], &v.body[..]);

            assert_eq!(0, v.player_bind_id);
            let player_bind = {
                let mut player = Player::new(1);
                player.set_bind(4);
                player
            };

            // test binded player
            assert!(v.bind_to_player(&player_bind).is_ok());
            assert_eq!(4, v.player_bind_id);

            //test unbinded player
            assert!(v.bind_to_player(&Player::new(32)).is_err());
            assert_eq!(4, v.player_bind_id);

            assert_eq!(
                &[1, 0, 9, 4, 4u8, 6, 0, 0, 0, 10, 0, 20, 0, 1, 2, 3, 4, 5, 6],
                &v.to_vangers_byte()[..]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn small_length__not_NID_VANGER() {
            assert!(Vanject::create_from_slice(&[]).is_err());
            assert!(Vanject::create_from_slice(&[1]).is_err());
            assert!(Vanject::create_from_slice(&[1, 0, 9, 4]).is_err());
            assert!(
                Vanject::create_from_slice(&[1, 0, 9, 4, 6, 0, 0, 0, 10, 0, 20, 0, 15, 0]).is_err()
            );
            assert!(
                Vanject::create_from_slice(&[1, 0, 9, 4, 6, 0, 0, 0, 10, 0, 20, 0, 15, 0, 111])
                    .is_ok()
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn correct__not_NID_VANGER() {
            let slice = std::iter::empty()
                .chain(&[1, 1, 1, 1]) // id = 16843009
                .chain(&[6, 0, 0, 0]) // time = 6
                .chain(&[10, 0]) // pos.x = 10
                .chain(&[20, 0]) // pos.y = 20
                .chain(&[15, 0]) // radius = 15
                .chain(&[1u8, 2, 3, 4, 5, 6])
                .map(|&b| b)
                .collect::<Vec<_>>();

            let v = Vanject::create_from_slice(&slice);
            assert!(v.is_ok());
            let mut v = v.unwrap();

            assert_eq!(16843009, v.id);
            assert_eq!(6, v.time);
            assert_eq!(10, v.pos.x);
            assert_eq!(20, v.pos.y);
            assert_eq!(15, v.radius);
            assert_eq!(&[1u8, 2, 3, 4, 5, 6], &v.body[..]);

            assert_eq!(0, v.player_bind_id);
            let player_bind = {
                let mut player = Player::new(1);
                player.set_bind(4);
                player
            };

            // test binded player
            assert!(v.bind_to_player(&player_bind).is_ok());
            assert_eq!(4, v.player_bind_id);

            //test unbinded player
            assert!(v.bind_to_player(&Player::new(32)).is_err());
            assert_eq!(4, v.player_bind_id);

            assert_eq!(
                &[1, 1, 1, 1, 4u8, 6, 0, 0, 0, 10, 0, 20, 0, 1, 2, 3, 4, 5, 6],
                &v.to_vangers_byte()[..]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn small_length__NID_VANGER() {
            assert!(Vanject::create_from_slice(&[]).is_err());
            assert!(Vanject::create_from_slice(&[1]).is_err());
            assert!(Vanject::create_from_slice(&[1, 1, 1, 1]).is_err());
            assert!(
                Vanject::create_from_slice(&[1, 1, 1, 1, 6, 0, 0, 0, 10, 0, 20, 0, 15]).is_err()
            );
            assert!(
                Vanject::create_from_slice(&[1, 1, 1, 1, 6, 0, 0, 0, 10, 0, 20, 0, 15, 0]).is_ok()
            );
        }
    }

    mod update_from_slice {
        use super::*;

        fn get_vanject(is_nid_vanger: bool) -> Vanject {
            let slice = std::iter::empty()
                .chain(if is_nid_vanger {
                    &[2, 0, 9, 4] // id = 67698690
                } else {
                    &[2, 1, 1, 1] // id = 16843010
                })
                .chain(&[6, 0, 0, 0]) // time = 6
                .chain(&[10, 0]) // pos.x = 10
                .chain(&[20, 0]) // pos.y = 20
                .chain(&[15, 0]) // radius = 15
                .chain(if is_nid_vanger {
                    &[8u8, 1u8, 2, 3, 4, 5, 6][..]
                } else {
                    &[1u8, 2, 3, 4, 5, 6][..]
                })
                .map(|&b| b)
                .collect::<Vec<_>>();

            Vanject::create_from_slice(&slice).unwrap()
        }

        #[test]
        #[allow(non_snake_case)]
        fn correct__NID_VANGER() {
            let mut v = get_vanject(true);
            v.player_bind_id = 9;

            let upd = std::iter::empty()
                .chain(&[2, 0, 9, 4]) // id = 67698690
                .chain(&[7, 0, 0, 0]) // time = 7
                .chain(&[11, 0]) // pos.x = 11
                .chain(&[21, 0]) // pos.y = 21
                .chain(&[8u8]) // y_half_size_of_screen (only inside NID::VANGER vajects)
                .chain(&[88]) // body = [88]
                .map(|&b| b)
                .collect::<Vec<_>>();

            assert!(v.update_from_slice(&upd).is_ok());

            assert_eq!(67698690, v.id);
            assert_eq!(7, v.time);
            assert_eq!(11, v.pos.x);
            assert_eq!(21, v.pos.y);
            assert_eq!(15, v.radius);
            assert_eq!(&[88], &v.body[..]);
            assert_eq!(9, v.player_bind_id);
        }

        #[test]
        #[allow(non_snake_case)]
        fn small_length__NID_VANGER() {
            let mut v = get_vanject(true);

            let id = v.id;
            let player_bind_id = v.player_bind_id;
            let time = v.time;
            let pos = v.pos;
            let radius = v.radius;
            let body = v.body.clone();

            let check = |v: &Vanject| {
                assert_eq!(id, v.id);
                assert_eq!(player_bind_id, v.player_bind_id);
                assert_eq!(time, v.time);
                assert_eq!(pos, v.pos);
                assert_eq!(radius, v.radius);
                assert_eq!(body, v.body);
            };

            assert!(v.update_from_slice(&[]).is_err());
            check(&v);
            assert!(v.update_from_slice(&[1]).is_err());
            check(&v);
            assert!(v.update_from_slice(&[2, 0, 9, 4]).is_err());
            check(&v);
            assert!(
                v.update_from_slice(&[2, 0, 9, 4, 6, 0, 0, 0, 10, 0, 20, 0])
                    .is_err()
            );
            check(&v);

            assert!(
                v.update_from_slice(&[2, 0, 9, 4, 7, 0, 0, 0, 11, 0, 21, 0, 8u8])
                    .is_ok()
            );
            assert_eq!(7, v.time);
            assert_eq!(Pos { x: 11, y: 21 }, v.pos);
            assert_eq!(radius, v.radius);
            assert_eq!(&[0u8][1..], &v.body[..]);

            assert!(
                v.update_from_slice(&[2, 0, 9, 4, 9, 0, 0, 0, 12, 0, 22, 0, 8u8, 110, 111])
                    .is_ok()
            );
            assert_eq!(9, v.time);
            assert_eq!(Pos { x: 12, y: 22 }, v.pos);
            assert_eq!(radius, v.radius);
            assert_eq!(&[110, 111], &v.body[..]);
        }

        #[test]
        #[allow(non_snake_case)]
        fn correct_not__NID_VANGER() {
            let mut v = get_vanject(false);
            v.player_bind_id = 9;

            let upd = std::iter::empty()
                .chain(&[2, 1, 1, 1]) // id = 16843010
                .chain(&[7, 0, 0, 0]) // time = 7
                .chain(&[11, 0]) // pos.x = 11
                .chain(&[21, 0]) // pos.y = 21
                .chain(&[88]) // body = [88]
                .map(|&b| b)
                .collect::<Vec<_>>();

            assert!(v.update_from_slice(&upd).is_ok());

            assert_eq!(16843010, v.id);
            assert_eq!(7, v.time);
            assert_eq!(11, v.pos.x);
            assert_eq!(21, v.pos.y);
            assert_eq!(15, v.radius);
            assert_eq!(&[88], &v.body[..]);
            assert_eq!(9, v.player_bind_id);
        }

        #[test]
        #[allow(non_snake_case)]
        fn small_length_not__NID_VANGER() {
            let mut v = get_vanject(false);

            let id = v.id;
            let player_bind_id = v.player_bind_id;
            let time = v.time;
            let pos = v.pos;
            let radius = v.radius;
            let body = v.body.clone();

            let check = |v: &Vanject| {
                assert_eq!(id, v.id);
                assert_eq!(player_bind_id, v.player_bind_id);
                assert_eq!(time, v.time);
                assert_eq!(pos, v.pos);
                assert_eq!(radius, v.radius);
                assert_eq!(body, v.body);
            };

            assert!(v.update_from_slice(&[]).is_err());
            check(&v);
            assert!(v.update_from_slice(&[1]).is_err());
            check(&v);
            assert!(v.update_from_slice(&[2, 1, 1, 1]).is_err());
            check(&v);
            assert!(
                v.update_from_slice(&[2, 1, 1, 1, 6, 0, 0, 0, 10, 0, 20])
                    .is_err()
            );
            check(&v);

            assert!(
                v.update_from_slice(&[2, 1, 1, 1, 9, 0, 0, 0, 11, 0, 21, 0])
                    .is_ok()
            );
            assert_eq!(9, v.time);
            assert_eq!(Pos { x: 11, y: 21 }, v.pos);
            assert_eq!(radius, v.radius);
            assert_eq!(&[0u8][1..], &v.body[..]);

            assert!(
                v.update_from_slice(&[2, 1, 1, 1, 11, 0, 0, 0, 12, 0, 22, 0, 111])
                    .is_ok()
            );
            assert_eq!(11, v.time);
            assert_eq!(Pos { x: 12, y: 22 }, v.pos);
            assert_eq!(radius, v.radius);
            assert_eq!(&[111], &v.body[..]);
        }

        #[test]
        fn update_with_missmatch_id() {
            let mut v = get_vanject(true);

            let id = v.id;
            let player_bind_id = v.player_bind_id;
            let time = v.time;
            let pos = v.pos;
            let radius = v.radius;
            let body = v.body.clone();

            let check = |v: &Vanject| {
                assert_eq!(id, v.id);
                assert_eq!(player_bind_id, v.player_bind_id);
                assert_eq!(time, v.time);
                assert_eq!(pos, v.pos);
                assert_eq!(radius, v.radius);
                assert_eq!(body, v.body);
            };

            assert!(
                v.update_from_slice(&[5, 5, 5, 5, 1, 0, 0, 0, 10, 0, 15, 0, 1, 1, 1, 1, 1, 1])
                    .is_err(),
            );

            check(&v);
        }
    }
}
