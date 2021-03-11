use chess5dlib::parse::test::read_and_parse_opt;
use chess5dlib::{
    prelude::*,
    eval::*,
    tree::*,
};
use std::time::{Duration, Instant};
use std::env;

// const DEPTH: usize = 3;
const MAX_BRANCHES: usize = 3;
const MAX_TIMELINES: usize = 8;
const TIMEOUT: u64 = 30;
const POOL_SIZE: usize = 1024;
const MAX_POOL_SIZE: usize = 100000;
const N_THREADS: u32 = 14;
const APPROX: bool = true;

fn main() {
    let path = env::args().last().unwrap();
    let mut game = read_and_parse_opt(&path).unwrap();

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
                .knight_opponent(-0.5)
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
                println!("{}. {} {{{:.2}}}", (turn / 2) + 1, node.path[0], value);
            } else {
                println!(" / {} {{{}}}", node.path[0], value);
            }

            #[cfg(feature = "countnodes")]
            {
                let nodes = *NODES.lock().unwrap();
                let sigma = *SIGMA.lock().unwrap();
                println!("{{N: {}, ms: {}, N/s: {:.4}}}", nodes, sigma.as_millis(), nodes as f64 / sigma.as_millis() as f64 * 1000.0);
            }

            if game.info.len_timelines() > MAX_TIMELINES {
                break
            }
        } else {
            break
        }
    }
}
