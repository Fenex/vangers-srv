use super::*;

#[repr(C)]
#[derive(Debug)]
pub struct MustodontStatistic {
    pub part_time1: i32,
    pub part_time2: i32,
    pub body_time: i32,
    pub make_time: i32,
}

impl PlayerStatistics for MustodontStatistic {
    fn get_struct_size() -> usize {
        4 * std::mem::size_of::<i32>()
    }
}

impl NetTransportSend for MustodontStatistic {
    fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.part_time1.to_le_bytes())
            .chain(&self.part_time2.to_le_bytes())
            .chain(&self.body_time.to_le_bytes())
            .chain(&self.make_time.to_le_bytes())
            .copied()
            .collect()
    }
}

impl NetTransportReceive for MustodontStatistic {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != std::mem::size_of::<Self>() {
            return Err("ERROR: missmatch size when try serialize MustodontStatistic").ok();
        }

        Some(Self {
            part_time1: slice_le_to_i32(&slice[0..4]),
            part_time2: slice_le_to_i32(&slice[4..8]),
            body_time: slice_le_to_i32(&slice[8..12]),
            make_time: slice_le_to_i32(&slice[12..16]),
        })
    }
}
