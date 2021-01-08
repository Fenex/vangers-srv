use crate::protocol::{NetTransportReceive, NetTransportSend};
use crate::utils::*;

use super::stats::{
    MechosomaStatistic as Mechosoma, MustodontStatistic as Mustodont,
    PassemblossStatistic as Passembloss, VanWarStatistics as VanWar,
};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Statistics {
    UNDEFINED,
    VanWar(VanWar),
    Mechosoma(Mechosoma),
    Passembloss(Passembloss),
    Huntage,
    Mustodont(Mustodont),
}

#[derive(Debug, Default)]
pub struct Body {
    kills: u8,
    deaths: u8,
    pub color: u8,
    world: u8,
    beebos: u32,
    rating: f32,
    car_index: u8,
    data1: i16,
    data2: i16,
    birth_time: u32,
    net_id: i32,
    // TODO: change `stats` to one of:
    //  - convert to <dyn PlayerStatistics>
    //  - use enum (see body_alt.rs)
    //  - something else?
    stats: Vec<u8>,
}

impl Body {
    /// Returns summary size of each field exclude `stats`.
    /// Not use `std::mem::size_of()` because of memory align.
    /// TODO: convert to derive proc_macro.
    fn get_base_struct_size() -> usize {
        1 + 1 + 1 + 1 + 4 + 4 + 1 + 2 + 2 + 4 + 4
    }
}

impl NetTransportSend for Body {
    fn to_vangers_byte(&self) -> Vec<u8> {
        vec![self.kills, self.deaths, self.color, self.world]
            .iter()
            .chain(&self.beebos.to_le_bytes())
            .chain(&self.rating.to_le_bytes())
            .chain(&[self.car_index])
            .chain(&self.data1.to_le_bytes())
            .chain(&self.data2.to_le_bytes())
            .chain(&self.birth_time.to_le_bytes())
            .chain(&self.net_id.to_le_bytes())
            .chain(&self.stats)
            .map(|&b| b)
            .collect::<Vec<_>>()
    }
}

impl NetTransportReceive for Body {
    fn from_slice(slice: &[u8]) -> Option<Self> {
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

        let stats = slice[25..].to_vec();

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
    fn body_from_slice_vanwar() {
        let mut data_base = get_body_base_slice();
        let stats = std::iter::empty()
            .chain(&20i32.to_le_bytes()) //max_live_time
            .chain(&21i32.to_le_bytes()) //min_live_time
            .chain(&22i32.to_le_bytes()) //kill_freq
            .chain(&23i32.to_le_bytes()) //death_freaq
            .map(|&b| b)
            .collect::<Vec<_>>();

        data_base.append(&mut stats.clone());

        let body = Body::from_slice(&data_base[..]);
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

        assert_eq!(&stats, &body.stats);
    }

    #[test]
    fn body_from_slice_mechosoma() {
        let mut data_base = get_body_base_slice();
        let stats = std::iter::empty()
            .chain(&20i32.to_le_bytes()) //item_count1
            .chain(&21i32.to_le_bytes()) //item_count2
            .chain(&22i32.to_le_bytes()) //max_transit_time
            .chain(&23i32.to_le_bytes()) //min_transit_time
            .chain(&24i32.to_le_bytes()) //sneak_count
            .chain(&25i32.to_le_bytes()) //lost_count
            .map(|&b| b)
            .collect::<Vec<_>>();

        data_base.append(&mut stats.clone());

        let body = Body::from_slice(&data_base[..]);
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

        assert_eq!(&stats, &body.stats);
    }

    #[test]
    fn body_from_slice_passembloss() {
        let mut data_base = get_body_base_slice();
        let stats = std::iter::empty()
            .chain(&20i32.to_le_bytes()) //total_time
            .chain(&21i32.to_le_bytes()) //checkpoint_lighting
            .chain(&22i32.to_le_bytes()) //min_time
            .chain(&23i32.to_le_bytes()) //max_time
            .map(|&b| b)
            .collect::<Vec<_>>();

        data_base.append(&mut stats.clone());

        let body = Body::from_slice(&data_base[..]);
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

        assert_eq!(&stats, &body.stats);
    }

    #[test]
    fn body_from_slice_mustodont() {
        let mut data_base = get_body_base_slice();
        let stats = std::iter::empty()
            .chain(&20i32.to_le_bytes()) //part_time1
            .chain(&21i32.to_le_bytes()) //part_time2
            .chain(&22i32.to_le_bytes()) //body_time
            .chain(&23i32.to_le_bytes()) //make_time
            .map(|&b| b)
            .collect::<Vec<_>>();

        data_base.append(&mut stats.clone());

        let body = Body::from_slice(&data_base[..]);
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

        assert_eq!(&stats, &body.stats);
    }

    #[test]
    fn body_from_slice_incorrect_size() {
        fn assert(data: &[u8]) {
            assert!(Body::from_slice(&data[..]).is_none());
            assert!(Body::from_slice(&data[..]).is_none());
            assert!(Body::from_slice(&data[..]).is_none());
            assert!(Body::from_slice(&data[..]).is_none());
        }

        assert(&[]);
        assert(&[1]);
        assert(&[1, 2, 3, 4, 5, 6]);
    }
}
