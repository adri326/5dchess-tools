//! Utility functions around finding threats and determining that a position is illegal

/// Functions to find threats
pub mod threat;
pub use threat::*;

/// Functions to eliminate moves
pub mod move_elim;
pub use move_elim::*;
