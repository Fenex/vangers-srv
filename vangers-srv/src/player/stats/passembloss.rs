use super::*;

#[repr(C)]
#[derive(Debug)]
pub struct PassemblossStatistic {
    pub total_time: i32, //hh:mm:ss
    pub checkpoint_lighting: i32,
    pub min_time: i32, //None
    pub max_time: i32, //None
}

impl PlayerStatistics for PassemblossStatistic {
    fn get_struct_size() -> usize {
        return 4 * std::mem::size_of::<i32>();
    }
}

impl NetTransportSend for PassemblossStatistic {
    fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.total_time.to_le_bytes())
            .chain(&self.checkpoint_lighting.to_le_bytes())
            .chain(&self.min_time.to_le_bytes())
            .chain(&self.max_time.to_le_bytes())
            .map(|&b| b)
            .collect()
    }
}

impl NetTransportReceive for PassemblossStatistic {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != std::mem::size_of::<Self>() {
            return Err("ERROR: missmatch size when try serialize PassemblossStatistic").ok();
        }

        Some(Self {
            total_time: slice_le_to_i32(&slice[0..4]),
            checkpoint_lighting: slice_le_to_i32(&slice[4..8]),
            min_time: slice_le_to_i32(&slice[8..12]),
            max_time: slice_le_to_i32(&slice[12..16]),
        })
    }
}
