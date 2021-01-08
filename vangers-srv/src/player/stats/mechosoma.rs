use super::*;

#[repr(C)]
#[derive(Debug)]
pub struct MechosomaStatistic {
    pub item_count1: i32,
    pub item_count2: i32,
    pub max_transit_time: i32, //hh:mm:ss - Время окончания гонки
    pub min_transit_time: i32, //None
    pub sneak_count: i32,      //Кол-во украденного товара
    pub lost_count: i32,       //Кол-во потерянного товара
}

impl PlayerStatistics for MechosomaStatistic {
    fn get_struct_size() -> usize {
        return 6 * std::mem::size_of::<i32>();
    }
}

impl NetTransportSend for MechosomaStatistic {
    fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.item_count1.to_le_bytes())
            .chain(&self.item_count2.to_le_bytes())
            .chain(&self.max_transit_time.to_le_bytes())
            .chain(&self.min_transit_time.to_le_bytes())
            .chain(&self.sneak_count.to_le_bytes())
            .chain(&self.lost_count.to_le_bytes())
            .map(|&b| b)
            .collect()
    }
}

impl NetTransportReceive for MechosomaStatistic {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != std::mem::size_of::<Self>() {
            return Err("ERROR: missmatch size when try serialize MechosomaStatistic").ok();
        }

        Some(Self {
            item_count1: slice_le_to_i32(&slice[0..4]),
            item_count2: slice_le_to_i32(&slice[4..8]),
            max_transit_time: slice_le_to_i32(&slice[8..12]),
            min_transit_time: slice_le_to_i32(&slice[12..16]),
            sneak_count: slice_le_to_i32(&slice[16..20]),
            lost_count: slice_le_to_i32(&slice[20..24]),
        })
    }
}
