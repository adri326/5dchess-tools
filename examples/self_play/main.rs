use chess5dlib::parse::test::read_and_parse_opt;
use chess5dlib::{
    prelude::*,
    eval::*,
    tree::*,
};
use std::time::{Duration, Instant};

// const DEPTH: usize = 3;
const MAX_BRANCHES: usize = 2;
const MAX_TIMELINES: usize = 6;
const TIMEOUT: u64 = 60;
const POOL_SIZE: usize = 1024;
const MAX_POOL_SIZE: usize = 100000;
const N_THREADS: u32 = 14;
const APPROX: bool = true;

fn main() {
    let mut game = read_and_parse_opt("tests/games/brawns-empty.json").unwrap();

    // let pg = no_partial_game(&game);
    // for b in pg.own_boards(&game) {
    //     println!("{:?}", b);
    // }

    for turn in 0..100 {
        if let Some((node, value)) = iddfs_bl_schedule(
            &game,
            MAX_BRANCHES,
            Some(Duration::new(TIMEOUT, 0)),
            PieceValues::default()
            .inactive_multiplier(0.05)
            .add(
                KingSafety::default()
                .diagonal_empty(0.0)
                .diagonal_opponent(-0.75)
                .orthogonal_empty(-0.25)
                .orthogonal_opponent(-1.0)
                .additional_king(-6.0)
            ).add(
                TimelineAdvantage::default()
            ).add(
                PawnProgression::default()
            ),
            POOL_SIZE,
            MAX_POOL_SIZE,
            N_THREADS,
            APPROX,
        ) {
            let new_partial_game = {
                let partial_game = no_partial_game(&game);
                node.path[0].generate_partial_game(&game, &partial_game).expect("Couldn't generate partial game!").flatten()
            };
            new_partial_game.apply(&mut game);

            if turn % 2 == 0 {
                println!("{}. {} {{{}}}", (turn / 2) + 1, node.path[0], value);
            } else {
                println!(" / {} {{{}}}", node.path[0], value);
            }

            if game.info.len_timelines() > MAX_TIMELINES {
                break
            }
        } else {
            break
        }
    }
}
