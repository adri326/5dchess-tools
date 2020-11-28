#[macro_use]
extern crate lazy_static;
extern crate scoped_threadpool;
extern crate permute;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;

pub mod game;
pub mod moves;
pub mod moveset;
pub mod resolve;
pub mod tree;
pub mod parse;
