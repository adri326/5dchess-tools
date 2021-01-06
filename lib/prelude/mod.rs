pub type Layer = isize;
pub type Time = isize;
pub type Physical = u8;
pub type Coords = (Layer, Time, Physical, Physical); // ⟨l, t, x, y⟩

pub mod game;
pub mod piece;
pub mod info;
pub mod board;
pub mod movement;

pub use game::*;
pub use piece::*;
pub use info::*;
pub use board::*;
pub use movement::*;
