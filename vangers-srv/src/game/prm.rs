use crate::utils::slice_le_to_i32;
// use std::convert::TryInto;

#[derive(Debug)]
pub struct VanWar {
    pub nascency: i32, // Bit-wise using
    pub team_mode: i32,
    pub world_access: i32, // 0 - all worlds, 1 - one world...
    pub max_kills: i32,
    pub max_time: u32,
}

impl VanWar {
    pub fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.nascency.to_le_bytes())
            .chain(&self.team_mode.to_le_bytes())
            .chain(&self.world_access.to_le_bytes())
            .chain(&self.max_kills.to_le_bytes())
            .chain(&(self.max_time as i32).to_le_bytes())
            .copied()
            .collect()
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        if slice.len() < std::mem::size_of::<Self>() {
            panic!("ERROR: size of given slice is too small to try serialize VanWar");
        }

        Self {
            nascency: slice_le_to_i32(&slice[0..4]),
            team_mode: slice_le_to_i32(&slice[4..8]),
            world_access: slice_le_to_i32(&slice[8..12]),
            max_kills: slice_le_to_i32(&slice[12..16]),
            // max_time: slice_le_to_i32(&slice[16..20]),
            max_time: i32::MAX as u32, // TODO: change to correct time set
        }
    }
}

impl Default for VanWar {
    fn default() -> Self {
        Self {
            nascency: 0,
            team_mode: 0,
            world_access: 0,
            max_kills: 100,
            max_time: 6000,
        }
    }
}

#[derive(Debug)]
pub struct Mechosoma {
    pub world: i32,
    pub product_quantity1: i32,
    pub product_quantity2: i32,
    pub one_at_a_time: i32,
    pub team_mode: i32,
}

impl Mechosoma {
    pub fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.world.to_le_bytes())
            .chain(&self.product_quantity1.to_le_bytes())
            .chain(&self.product_quantity2.to_le_bytes())
            .chain(&self.one_at_a_time.to_le_bytes())
            .chain(&self.team_mode.to_le_bytes())
            .copied()
            .collect()
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        if slice.len() < std::mem::size_of::<Self>() {
            panic!("ERROR: size of given slice is too small to try serialize Mechosoma");
        }

        Self {
            world: slice_le_to_i32(&slice[0..4]),
            product_quantity1: slice_le_to_i32(&slice[4..8]),
            product_quantity2: slice_le_to_i32(&slice[8..12]),
            one_at_a_time: slice_le_to_i32(&slice[12..16]),
            team_mode: slice_le_to_i32(&slice[16..20]),
        }
    }
}

impl Default for Mechosoma {
    fn default() -> Self {
        Self {
            world: 0,
            product_quantity1: 10,
            product_quantity2: 10,
            one_at_a_time: 10,
            team_mode: 0,
        }
    }
}

#[derive(Debug)]
pub struct Passembloss {
    pub checkpoints_number: i32,
    pub random_escave: i32,
}

impl Passembloss {
    pub fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.checkpoints_number.to_le_bytes())
            .chain(&self.random_escave.to_le_bytes())
            .copied()
            .collect()
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        if slice.len() < std::mem::size_of::<Self>() {
            panic!("ERROR: size of given slice is too small to try serialize Passembloss");
        }

        Self {
            checkpoints_number: slice_le_to_i32(&slice[0..4]),
            random_escave: slice_le_to_i32(&slice[4..8]),
        }
    }
}

impl Default for Passembloss {
    fn default() -> Self {
        Self {
            checkpoints_number: 10,
            random_escave: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct Mustodont {
    pub unique_mechos_name: i32,
    pub team_mode: i32,
}

impl Mustodont {
    pub fn to_vangers_byte(&self) -> Vec<u8> {
        std::iter::empty()
            .chain(&self.unique_mechos_name.to_le_bytes())
            .chain(&self.team_mode.to_le_bytes())
            .copied()
            .collect()
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        if slice.len() < std::mem::size_of::<Self>() {
            panic!("ERROR: size of given slice is too small to try serialize Mustodont");
        }

        Self {
            unique_mechos_name: slice_le_to_i32(&slice[0..4]),
            team_mode: slice_le_to_i32(&slice[4..8]),
        }
    }
}
