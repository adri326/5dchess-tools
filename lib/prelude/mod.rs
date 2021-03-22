pub type Layer = isize;
pub type Time = isize;
pub type Physical = i8;

/// Contains the `Board` structure and operations on pieces within a board
pub mod board;
pub use board::*;

/// Contains the `BitBoard` and `BitBoardMask` structures, various operations on bitboards and pre-computed bitboards and masks
pub mod bitboard;
pub use bitboard::*;

/// Contains the `Coords` struct with its associated functions
pub mod coords;
pub use coords::*;

/// Contains the `Game` struct and the `BoardArray` struct, with their respective, associated functions
pub mod game;
pub use game::*;

/// Contains the `Info` and `TimelineInfo` structs
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
