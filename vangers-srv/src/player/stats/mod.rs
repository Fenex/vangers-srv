mod mechosoma;
mod mustodont;
mod passembloss;
mod vanwar;

pub use mechosoma::*;
pub use mustodont::*;
pub use passembloss::*;
pub use vanwar::*;

use crate::protocol::{NetTransport, NetTransportReceive, NetTransportSend};
use crate::utils::slice_le_to_i32;

pub trait PlayerStatistics: NetTransport {
    fn get_struct_size() -> usize;
}
