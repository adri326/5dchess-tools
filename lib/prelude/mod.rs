pub type Layer = isize;
pub type Time = isize;
pub type Physical = u8;

pub mod game;
pub use game::*;

pub mod piece;
pub use piece::*;

pub mod tile;
pub use tile::*;

pub mod info;
pub use info::*;

pub mod board;
pub use board::*;

pub mod movement;
pub use movement::*;

pub mod coords;
pub use coords::*;

pub mod partial_state;
pub use partial_state::*;
