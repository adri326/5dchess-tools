use chess5dlib::parse::test::read_and_parse_opt;
use chess5dlib::{
    prelude::*,
    gen::*,
    mate::*,
    eval::*,
    tree::*,
};
use rand::{Rng, prelude::SliceRandom};
use std::fs::read_dir;
use std::time::{Duration, Instant};
use std::path::Path;
use std::borrow::Cow;

const DEPTH: usize = 4;
const MAX_BRANCHES: usize = 2;
const TIMEOUT: u64 = 60;
const POOL_SIZE: usize = 256;
const N_THREADS: u32 = 14;

fn main() {
    let mut game = read_and_parse_opt("tests/games/standard-empty.json").unwrap();

    for turn in 0..20 {
        if let Some((node, value)) = dfs_bl_schedule(&game, DEPTH, MAX_BRANCHES, Some(Duration::new(TIMEOUT, 0)), NoEvalFn::new(), POOL_SIZE, N_THREADS) {
            let new_partial_game = {
                let partial_game = no_partial_game(&game);
                node.path[0].generate_partial_game(&game, &partial_game).expect("Couldn't generate partial game!").flatten()
            };
            new_partial_game.apply(&mut game);

            // println!("{:?}", game);

            println!("turn {}: {}", turn, node.path[0]);
        }
    }
}
