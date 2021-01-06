pub type Layer = isize;
pub type Time = isize;
pub type Physical = u8;

pub mod game;
pub mod piece;
pub mod info;
pub mod board;

pub use game::*;
pub use piece::*;
pub use info::*;
pub use board::*;
