use crate::player::stats::IBodyStats;

use crate::game;
use crate::utils::{slice_le_to_i16, slice_le_to_i32, slice_le_to_u32};

use super::stats::{
    MechosomaStatistic as Mechosoma, MustodontStatistic as Mustodont,
    PassemblossStatistic as Passembloss, VanWarStatistics as VanWar,
};

#[derive(Debug)]
pub enum Statistics {
    UNDEFINED,
    VanWar(VanWar),
    Mechosoma(Mechosoma),
    Passembloss(Passembloss),
    Huntage,
    Mustodont(Mustodont),
}

pub struct Body {
    kills: u8,
    deaths: u8,
    color: u8,
    world: u8,
    beebos: u32,
    rating: f32,
    car_index: u8,
    data1: i16,
    data2: i16,
    birth_time: u32,
    net_id: i32,
    stats: Statistics,
}

impl Body {
    /// Returns summary size of each field exclude `stats`.
    /// Not use `std::mem::size_of()` because of memory align.
    /// TODO: convert to derive proc_macro.
    fn get_base_struct_size() -> usize {
        1 + 1 + 1 + 1 + 4 + 4 + 1 + 2 + 2 + 4 + 4
    }

    pub fn to_vangers_byte(&self) -> Vec<u8> {
        let body = vec![self.kills, self.deaths, self.color, self.world];
        let mut body = body
            .iter()
            .chain(&self.beebos.to_le_bytes())
            .chain(&self.rating.to_le_bytes())
            .chain(&[self.car_index])
            .chain(&self.data1.to_le_bytes())
            .chain(&self.data2.to_le_bytes())
            .chain(&self.birth_time.to_le_bytes())
            .chain(&self.net_id.to_le_bytes())
            .map(|&b| b)
            .collect::<Vec<_>>();

        let mut body_stats = match &self.stats {
            Statistics::VanWar(e) => e.to_vangers_byte(),
            Statistics::Mechosoma(e) => e.to_vangers_byte(),
            Statistics::Passembloss(e) => e.to_vangers_byte(),
            Statistics::Mustodont(e) => e.to_vangers_byte(),
            _ => vec![],
        };

        body.append(&mut body_stats);
        body
    }

    pub fn from_slice(gametype: game::Type, slice: &[u8]) -> Option<Self> {
        let base_count_bytes = Self::get_base_struct_size();
        if slice.len() < base_count_bytes {
            return None;
        }

        let kills = slice[0];
        let deaths = slice[1];
        let color = slice[2];
        let world = slice[3];
        let beebos = slice_le_to_u32(&slice[4..8]);

        let mut rating = [0u8, 0, 0, 0];
        rating.copy_from_slice(&slice[8..12]);
        let rating = f32::from_le_bytes(rating);

        let car_index = slice[12];
        let data1 = slice_le_to_i16(&slice[13..15]);
        let data2 = slice_le_to_i16(&slice[15..17]);
        let birth_time = slice_le_to_u32(&slice[17..21]);
        let net_id = slice_le_to_i32(&slice[21..25]);

        let stats = match gametype {
            game::Type::VAN_WAR => Statistics::VanWar(VanWar::from_slice(&slice[25..])),
            game::Type::MECHOSOMA => Statistics::Mechosoma(Mechosoma::from_slice(&slice[25..])),
            game::Type::PASSEMBLOSS => {
                Statistics::Passembloss(Passembloss::from_slice(&slice[25..]))
            }
            game::Type::HUNTAGE => Statistics::Huntage,
            game::Type::MUSTODONT => Statistics::Mustodont(Mustodont::from_slice(&slice[25..])),
            _ => return None,
        };

        Some(Self {
            kills,
            deaths,
            color,
            world,
            beebos,
            rating,
            car_index,
            data1,
            data2,
            birth_time,
            net_id,
            stats,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::Type as GameType;

    fn get_body_base_slice() -> Vec<u8> {
        vec![1u8, 2u8, 3u8, 4u8]
            .iter() // kills, deaths, color, world
            .chain(&5u32.to_le_bytes()) //beebos
            .chain(&6f32.to_le_bytes()) //rating
            .chain(&[7u8]) //car_index
            .chain(&8i16.to_le_bytes()) //data1
            .chain(&9i16.to_le_bytes()) //data2
            .chain(&10u32.to_le_bytes()) //birth_time
            .chain(&11u32.to_le_bytes()) //net_id
            .map(|&b| b)
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_body_from_slice_vanwar() {
        let data_base = get_body_base_slice();
        let data = data_base
            .iter()
            .chain(&20i32.to_le_bytes()) //max_live_time
            .chain(&21i32.to_le_bytes()) //min_live_time
            .chain(&22i32.to_le_bytes()) //kill_freq
            .chain(&23i32.to_le_bytes()) //death_freaq
            .map(|&b| b)
            .collect::<Vec<_>>();

        let body = Body::from_slice(GameType::VAN_WAR, &data[..]);
        assert!(body.is_some());
        let body = body.unwrap();

        assert_eq!(1, body.kills);
        assert_eq!(2, body.deaths);
        assert_eq!(3, body.color);
        assert_eq!(4, body.world);
        assert_eq!(5, body.beebos);
        assert_eq!(6.0, body.rating);
        assert_eq!(7, body.car_index);
        assert_eq!(8, body.data1);
        assert_eq!(9, body.data2);
        assert_eq!(10, body.birth_time);
        assert_eq!(11, body.net_id);

        match body.stats {
            Statistics::VanWar(s) => {
                assert_eq!(20, s.max_live_time);
                assert_eq!(21, s.min_live_time);
                assert_eq!(22, s.kill_freq);
                assert_eq!(23, s.death_freaq);
            }
            gtype => panic!(
                "missmatch gametype, expected: `VanWar`, actual: {:?}",
                gtype
            ),
        }
    }

    #[test]
    fn test_body_from_slice_mechosoma() {
        let data_base = get_body_base_slice();
        let data = data_base
            .iter()
            .chain(&20i32.to_le_bytes()) //item_count1
            .chain(&21i32.to_le_bytes()) //item_count2
            .chain(&22i32.to_le_bytes()) //max_transit_time
            .chain(&23i32.to_le_bytes()) //min_transit_time
            .chain(&24i32.to_le_bytes()) //sneak_count
            .chain(&25i32.to_le_bytes()) //lost_count
            .map(|&b| b)
            .collect::<Vec<_>>();

        let body = Body::from_slice(GameType::MECHOSOMA, &data[..]);
        assert!(body.is_some());
        let body = body.unwrap();

        assert_eq!(1, body.kills);
        assert_eq!(2, body.deaths);
        assert_eq!(3, body.color);
        assert_eq!(4, body.world);
        assert_eq!(5, body.beebos);
        assert_eq!(6.0, body.rating);
        assert_eq!(7, body.car_index);
        assert_eq!(8, body.data1);
        assert_eq!(9, body.data2);
        assert_eq!(10, body.birth_time);
        assert_eq!(11, body.net_id);

        match body.stats {
            Statistics::Mechosoma(s) => {
                assert_eq!(20, s.item_count1);
                assert_eq!(21, s.item_count2);
                assert_eq!(22, s.max_transit_time);
                assert_eq!(23, s.min_transit_time);
                assert_eq!(24, s.sneak_count);
                assert_eq!(25, s.lost_count);
            }
            gtype => panic!(
                "missmatch gametype, expected: `Mechosoma`, actual: {:?}",
                gtype
            ),
        }
    }

    #[test]
    fn test_body_from_slice_passembloss() {
        let data_base = get_body_base_slice();
        let data = data_base
            .iter()
            .chain(&20i32.to_le_bytes()) //total_time
            .chain(&21i32.to_le_bytes()) //checkpoint_lighting
            .chain(&22i32.to_le_bytes()) //min_time
            .chain(&23i32.to_le_bytes()) //max_time
            .map(|&b| b)
            .collect::<Vec<_>>();

        let body = Body::from_slice(GameType::PASSEMBLOSS, &data[..]);
        assert!(body.is_some());
        let body = body.unwrap();

        assert_eq!(1, body.kills);
        assert_eq!(2, body.deaths);
        assert_eq!(3, body.color);
        assert_eq!(4, body.world);
        assert_eq!(5, body.beebos);
        assert_eq!(6.0, body.rating);
        assert_eq!(7, body.car_index);
        assert_eq!(8, body.data1);
        assert_eq!(9, body.data2);
        assert_eq!(10, body.birth_time);
        assert_eq!(11, body.net_id);

        match body.stats {
            Statistics::Passembloss(s) => {
                assert_eq!(20, s.total_time);
                assert_eq!(21, s.checkpoint_lighting);
                assert_eq!(22, s.min_time);
                assert_eq!(23, s.max_time);
            }
            gtype => panic!(
                "missmatch gametype, expected: `Passembloss`, actual: {:?}",
                gtype
            ),
        }
    }

    #[test]
    fn test_body_from_slice_mustodont() {
        let data_base = get_body_base_slice();
        let data = data_base
            .iter()
            .chain(&20i32.to_le_bytes()) //part_time1
            .chain(&21i32.to_le_bytes()) //part_time2
            .chain(&22i32.to_le_bytes()) //body_time
            .chain(&23i32.to_le_bytes()) //make_time
            .map(|&b| b)
            .collect::<Vec<_>>();

        let body = Body::from_slice(GameType::MUSTODONT, &data[..]);
        assert!(body.is_some());
        let body = body.unwrap();

        assert_eq!(1, body.kills);
        assert_eq!(2, body.deaths);
        assert_eq!(3, body.color);
        assert_eq!(4, body.world);
        assert_eq!(5, body.beebos);
        assert_eq!(6.0, body.rating);
        assert_eq!(7, body.car_index);
        assert_eq!(8, body.data1);
        assert_eq!(9, body.data2);
        assert_eq!(10, body.birth_time);
        assert_eq!(11, body.net_id);

        match body.stats {
            Statistics::Mustodont(s) => {
                assert_eq!(20, s.part_time1);
                assert_eq!(21, s.part_time2);
                assert_eq!(22, s.body_time);
                assert_eq!(23, s.make_time);
            }
            gtype => panic!(
                "missmatch gametype, expected: `Mustodont`, actual: {:?}",
                gtype
            ),
        }
    }

    #[test]
    fn test_body_from_slice_incorrect_size() {
        fn assert(data: &[u8]) {
            assert!(Body::from_slice(GameType::VAN_WAR, &data[..]).is_none());
            assert!(Body::from_slice(GameType::MECHOSOMA, &data[..]).is_none());
            assert!(Body::from_slice(GameType::PASSEMBLOSS, &data[..]).is_none());
            assert!(Body::from_slice(GameType::MUSTODONT, &data[..]).is_none());
        }

        assert(&[]);
        assert(&[1]);
        assert(&[1, 2, 3, 4, 5, 6]);
        assert(&Vec::with_capacity(1000));
    }
}
