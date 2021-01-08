mod config;
mod game;
mod prm;
mod world;

pub use config::*;
pub use game::*;
pub use prm::*;
pub use world::*;

use enum_primitive_derive::Primitive;

const UNCONFIGURED: ::std::os::raw::c_char = -1;
const VAN_WAR: ::std::os::raw::c_char = 0;
const MECHOSOMA: ::std::os::raw::c_char = 1;
const PASSEMBLOSS: ::std::os::raw::c_char = 2;
const MIR_RAGE: ::std::os::raw::c_char = 3;
// const NUMBER_MP_GAMES: ::std::os::raw::c_char = 3; //unnecessary in Rust (?)
const HUNTAGE: ::std::os::raw::c_char = 4;
const MUSTODONT: ::std::os::raw::c_char = 5;

#[allow(non_camel_case_types)]
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Type {
    UNCONFIGURED = UNCONFIGURED as isize,
    VAN_WAR = VAN_WAR as isize,
    MECHOSOMA = MECHOSOMA as isize,
    PASSEMBLOSS = PASSEMBLOSS as isize,
    MIR_RAGE = MIR_RAGE as isize,
    // NUMBER_MP_GAMES = NUMBER_MP_GAMES as isize, //unnecessary
    HUNTAGE = HUNTAGE as isize, // Some people're still using it.
    MUSTODONT = MUSTODONT as isize,
}
