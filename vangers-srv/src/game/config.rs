use std::mem::size_of;

use ::num_traits::FromPrimitive;

use crate::protocol::{NetTransportReceive, NetTransportSend};
use crate::utils::slice_le_to_i32;

use super::prm::*;
use super::Type;

#[derive(Debug)]
pub enum GameMode {
    VanWar(VanWar),
    Mechosoma(Mechosoma),
    Passembloss(Passembloss),
    MirRage,
    Huntage,
    Mustodont(Mustodont),
}

#[derive(Debug)]
pub struct Config {
    pub initial_rnd: i32,
    pub initial_cash: i32,
    // pub game_type: i32, //in Rust covered by game_type below
    pub artefacts_using: i32,
    pub in_escave_time: i32,
    pub color: i32,
    pub game_type: GameMode,
}

impl Config {
    pub fn get_gametype(&self) -> Type {
        match self.game_type {
            GameMode::VanWar(_) => Type::VAN_WAR,
            GameMode::Mechosoma(_) => Type::MECHOSOMA,
            GameMode::Passembloss(_) => Type::PASSEMBLOSS,
            GameMode::MirRage => Type::MIR_RAGE,
            GameMode::Huntage => Type::HUNTAGE,
            GameMode::Mustodont(_) => Type::MUSTODONT,
        }
    }

    #[cfg(test)]
    pub fn new(gametype: Type) -> Self {
        let game_type = match gametype {
            Type::VAN_WAR => GameMode::VanWar(VanWar::default()),
            Type::MECHOSOMA => GameMode::Mechosoma(Mechosoma::default()),
            Type::PASSEMBLOSS => GameMode::Passembloss(Passembloss::default()),
            Type::MIR_RAGE => GameMode::MirRage,
            Type::HUNTAGE => GameMode::Huntage,
            Type::MUSTODONT => GameMode::Mustodont(Mustodont::default()),
            Type::UNCONFIGURED => panic!("Try to create `GameType` based on unconfigured game"),
        };

        Self {
            initial_rnd: 1,
            initial_cash: 100000,
            artefacts_using: 0,
            in_escave_time: 60,
            color: 0,
            game_type,
        }
    }
}

impl NetTransportSend for Config {
    fn to_vangers_byte(&self) -> Vec<u8> {
        let mut srv_data = std::iter::empty()
            .chain(&self.initial_rnd.to_le_bytes())
            .chain(&(self.get_gametype() as i32).to_le_bytes())
            .chain(&self.initial_cash.to_le_bytes())
            .chain(&self.artefacts_using.to_le_bytes())
            .chain(&self.in_escave_time.to_le_bytes())
            .chain(&self.color.to_le_bytes())
            .map(|&b| b)
            .collect::<Vec<_>>();

        let mut game_mode = match &self.game_type {
            // TODO: change to generic <dyn Trait> ...
            GameMode::VanWar(e) => e.to_vangers_byte(),
            GameMode::Mechosoma(e) => e.to_vangers_byte(),
            GameMode::Passembloss(e) => e.to_vangers_byte(),
            GameMode::Mustodont(e) => e.to_vangers_byte(),
            GameMode::Huntage | GameMode::MirRage => vec![],
        };

        srv_data.append(&mut game_mode);
        srv_data
    }
}

impl NetTransportReceive for Config {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        let base_count_bytes = {
            const BASE_FIELDS_COUNT: usize = 6;
            match BASE_FIELDS_COUNT * size_of::<i32>() {
                size if slice.len() >= size => size,
                _ => return None,
            }
        };

        let initial_rnd = slice_le_to_i32(&slice[0..4]);
        let game_type = slice_le_to_i32(&slice[4..8]);
        let initial_cash = slice_le_to_i32(&slice[8..12]);
        let artefacts_using = slice_le_to_i32(&slice[12..16]);
        let in_escave_time = slice_le_to_i32(&slice[16..20]);
        let color = slice_le_to_i32(&slice[20..24]);

        let game_type = match (Type::from_i32(game_type), slice.len() - base_count_bytes) {
            (Some(gt), len) if gt == Type::VAN_WAR && len >= size_of::<VanWar>() => {
                Some(GameMode::VanWar(VanWar::from_slice(&slice[24..])))
            }
            (Some(gt), len) if gt == Type::MECHOSOMA && len >= size_of::<Mechosoma>() => {
                Some(GameMode::Mechosoma(Mechosoma::from_slice(&slice[24..])))
            }
            (Some(gt), len) if gt == Type::PASSEMBLOSS && len >= size_of::<Passembloss>() => {
                Some(GameMode::Passembloss(Passembloss::from_slice(&slice[24..])))
            }
            (Some(gt), len) if gt == Type::MUSTODONT && len >= size_of::<Mustodont>() => {
                Some(GameMode::Mustodont(Mustodont::from_slice(&slice[24..])))
            }
            (Some(gt), _) if gt == Type::HUNTAGE => Some(GameMode::Huntage),
            (Some(gt), _) if gt == Type::MIR_RAGE => Some(GameMode::MirRage),
            _ => None,
        };

        game_type.and_then(|game_type| {
            Some(Self {
                initial_rnd,
                initial_cash,
                artefacts_using,
                in_escave_time,
                color,
                game_type,
            })
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::slice_i32_to_vec_u8;

    #[test]
    fn serverdata_from_slice_wanwar_0() {
        let data = [1, Type::VAN_WAR as i32, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        // check for too small slice
        assert!(Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 2])).is_none());

        let sd = Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 1]));
        assert!(sd.is_some());
        let sd = sd.unwrap();

        assert_eq!(1, sd.initial_rnd);
        assert_eq!(3, sd.initial_cash);
        assert_eq!(4, sd.artefacts_using);
        assert_eq!(5, sd.in_escave_time);
        assert_eq!(6, sd.color);
        assert_eq!(Type::VAN_WAR, sd.get_gametype());

        match sd.game_type {
            GameMode::VanWar(s) => {
                assert_eq!(7, s.nascency);
                assert_eq!(8, s.team_mode);
                assert_eq!(9, s.world_access);
                assert_eq!(10, s.max_kills);
                // TODO: do not forget uncomment the line when max_time will be fixed
                // assert_eq!(11, s.max_time);
            }
            gm => panic!("missmatch gamemode, expected: `WanWar`, actual: {:?}", gm),
        };
    }

    #[test]
    fn serverdata_from_slice_mechosoma_1() {
        let data = [1, Type::MECHOSOMA as i32, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        // check for too small slice
        assert!(Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 2])).is_none());

        let sd = Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 1]));
        assert!(sd.is_some());
        let sd = sd.unwrap();

        assert_eq!(1, sd.initial_rnd);
        assert_eq!(3, sd.initial_cash);
        assert_eq!(4, sd.artefacts_using);
        assert_eq!(5, sd.in_escave_time);
        assert_eq!(6, sd.color);
        assert_eq!(Type::MECHOSOMA, sd.get_gametype());

        match sd.game_type {
            GameMode::Mechosoma(s) => {
                assert_eq!(7, s.world);
                assert_eq!(8, s.product_quantity1);
                assert_eq!(9, s.product_quantity2);
                assert_eq!(10, s.one_at_a_time);
                assert_eq!(11, s.team_mode);
            }
            gm => panic!(
                "missmatch gamemode, expected: `Mechosoma`, actual: {:?}",
                gm
            ),
        };
    }

    #[test]
    fn serverdata_from_slice_passembloss_2() {
        let data = [
            1i32,
            Type::PASSEMBLOSS as i32,
            3i32,
            4i32,
            5i32,
            6i32,
            7,
            8,
            9,
        ];

        // check for too small slice
        assert!(Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 2])).is_none());

        let sd = Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 1]));
        assert!(sd.is_some());
        let sd = sd.unwrap();

        assert_eq!(1, sd.initial_rnd);
        assert_eq!(3, sd.initial_cash);
        assert_eq!(4, sd.artefacts_using);
        assert_eq!(5, sd.in_escave_time);
        assert_eq!(6, sd.color);
        assert_eq!(Type::PASSEMBLOSS, sd.get_gametype());

        match sd.game_type {
            GameMode::Passembloss(s) => {
                assert_eq!(7, s.checkpoints_number);
                assert_eq!(8, s.random_escave);
            }
            gm => panic!(
                "missmatch gamemode, expected: `Passembloss`, actual: {:?}",
                gm
            ),
        };
    }

    #[test]
    fn serverdata_from_slice_huntage_3() {
        let data = [1i32, Type::HUNTAGE as i32, 3i32, 4i32, 5i32, 6i32, 7];

        // check for too small slice
        assert!(Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 2])).is_none());

        let sd = Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 1]));
        assert!(sd.is_some());
        let sd = sd.unwrap();

        assert_eq!(1, sd.initial_rnd);
        assert_eq!(3, sd.initial_cash);
        assert_eq!(4, sd.artefacts_using);
        assert_eq!(5, sd.in_escave_time);
        assert_eq!(6, sd.color);
        assert_eq!(Type::HUNTAGE, sd.get_gametype());
    }

    #[test]
    fn serverdata_from_slice_mustodont_4() {
        let data = [
            1i32,
            Type::MUSTODONT as i32,
            3i32,
            4i32,
            5i32,
            6i32,
            7,
            8,
            9,
        ];

        // check for too small slice
        assert!(Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 2])).is_none());

        let sd = Config::from_slice(&slice_i32_to_vec_u8(&data[0..data.len() - 1]));
        assert!(sd.is_some());
        let sd = sd.unwrap();

        assert_eq!(1, sd.initial_rnd);
        assert_eq!(3, sd.initial_cash);
        assert_eq!(4, sd.artefacts_using);
        assert_eq!(5, sd.in_escave_time);
        assert_eq!(6, sd.color);
        assert_eq!(Type::MUSTODONT, sd.get_gametype());

        match sd.game_type {
            GameMode::Mustodont(s) => {
                assert_eq!(7, s.unique_mechos_name);
                assert_eq!(8, s.team_mode);
            }
            gm => panic!(
                "missmatch gamemode, expected: `Mustodont`, actual: {:?}",
                gm
            ),
        };
    }

    #[test]
    fn serverdata_from_slice_undefined_gametype() {
        let data = [1i32, -1, 3i32, 4i32, 5i32, 6i32, 7, 8];
        assert!(Config::from_slice(&slice_i32_to_vec_u8(&data)).is_none());

        let data = [1i32, 6, 3i32, 4i32, 5i32, 6i32, 7, 8];
        assert!(Config::from_slice(&slice_i32_to_vec_u8(&data)).is_none());
    }

    #[test]
    fn passembloss_real_example() {
        let slice = [
            207, 204, 22, 84, 2, 0, 0, 0, 160, 134, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let cfg = Config::from_slice(&slice);

        assert!(cfg.is_some());
        let cfg = cfg.unwrap();

        assert_eq!(slice_le_to_i32(&slice[0..4]), cfg.initial_rnd);
        assert_eq!(Type::PASSEMBLOSS, cfg.get_gametype());
        assert_eq!(2i32, cfg.get_gametype() as i32);
        assert_eq!(100000, cfg.initial_cash);
        assert_eq!(0, cfg.artefacts_using);
        if let GameMode::Passembloss(Passembloss {
            checkpoints_number,
            random_escave,
        }) = cfg.game_type
        {
            assert_eq!(10, checkpoints_number);
            assert_eq!(0, random_escave);
        } else {
            panic!(
                "`cfg.gametype` should be Passembloss, actual is: `{:?}`",
                cfg.game_type
            );
        }
    }
}
