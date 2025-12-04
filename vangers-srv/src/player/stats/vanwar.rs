use super::*;

#[repr(C)]
#[derive(Debug)]
pub struct VanWarStatistics {
    pub max_live_time: i32, // None
    pub min_live_time: i32, // None
    pub kill_freq: i32,     // hh:mm:ss - Средний период убийств
    pub death_freaq: i32,   // hh:mm:ss - Средний период смертей
}

impl PlayerStatistics for VanWarStatistics {
    fn get_struct_size() -> usize {
        4 * std::mem::size_of::<i32>()
    }
}

impl NetTransportSend for VanWarStatistics {
    fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.max_live_time.to_le_bytes())
            .chain(&self.min_live_time.to_le_bytes())
            .chain(&self.kill_freq.to_le_bytes())
            .chain(&self.death_freaq.to_le_bytes())
            .copied()
            .collect()
    }
}

impl NetTransportReceive for VanWarStatistics {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != std::mem::size_of::<Self>() {
            return Err("ERROR: missmatch size when try serialize VanWarStatistics").ok();
        }

        Some(Self {
            max_live_time: slice_le_to_i32(&slice[0..4]),
            min_live_time: slice_le_to_i32(&slice[4..8]),
            kill_freq: slice_le_to_i32(&slice[8..12]),
            death_freaq: slice_le_to_i32(&slice[12..16]),
        })
    }
}
