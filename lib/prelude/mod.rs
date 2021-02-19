pub type Layer = isize;
pub type Time = isize;
pub type Physical = i8;

pub mod board;
pub use board::*;

pub mod bitboard;
pub use bitboard::*;

pub mod coords;
pub use coords::*;

pub mod game;
pub use game::*;

pub mod info;
pub use info::*;

pub mod piece;
pub use piece::*;

pub mod movement;
pub use movement::*;

pub mod partial_game;
pub use partial_game::*;

pub mod goal;
pub use goal::*;

pub mod tile;
pub use tile::*;

pub mod time;
pub use time::*;
