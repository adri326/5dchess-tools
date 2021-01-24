#[macro_use]
extern crate lazy_static;
extern crate colored;
extern crate scoped_threadpool;
extern crate serde;
extern crate serde_json;
extern crate itertools;

pub mod prelude;
pub use prelude::*;

pub mod parse;
pub mod traversal;
pub mod strategies;
pub mod goals;
pub mod utils;
