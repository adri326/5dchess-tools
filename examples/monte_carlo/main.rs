extern crate chrono;

use chess5dlib::parse::test::read_and_parse;
use chess5dlib::random::*;
use chess5dlib::*;
use chrono::Utc;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;
use std::time::{Duration, Instant};

pub mod bench;

/**
    A piece of code that chooses and does random moves, to then submit it to 5d-chess-db.
**/
fn main() -> std::io::Result<()> {
    let performance = bench::benchmark();

    let game = read_and_parse("tests/games/standard-empty.json");

    loop {
        let timestamp = Utc::now();
        let mut header = String::with_capacity(1024);
        let mut res = String::with_capacity(1024);
        let today = Utc::today();
        header += "[Board \"Standard\"]\n";
        header += "[Variant \"Standard\"]\n";
        header += "[Mode \"5D\"]\n";
        header += &format!("[Date \"{}\"]\n", today.format("%Y.%m.%d"));
        header += "[White \"5D-Chess-DB-Gen_5dchess-tools-v2\"]\n";
        header += "[Black \"5D-Chess-DB-Gen_5dchess-tools-v2\"]\n";
        header += "[5DChess_tools \"^0.2.0\"]\n";
        header += &format!(
            "[Bench_time \"{}\"]\n",
            performance.as_nanos() as f64 / 1000000.0
        );
        let mut partial_game = no_partial_game(&game);
        let mut stopped = false;
        let mut result: Option<RandomLegalMovesetReason> = None;
        let start = Instant::now();

        // Max limit of 100 turns
        for turn in 0..50 {
            if start.elapsed() > Duration::new(60, 0) {
                break;
            }

            let checkmate_start = Instant::now();
            match random_legal_moveset(&game, &partial_game, Some(Duration::new(30, 0))) {
                Ok((ms, new_partial_game)) => {
                    // Move found
                    let new_partial_game = new_partial_game.flatten();
                    partial_game = new_partial_game;
                    if turn & 1 == 0 {
                        res += &format!("\n{}. ", turn / 2 + 1);
                    } else {
                        res += "/ ";
                    }
                    res += &format!("{}", ms);
                }
                Err(reason) => {
                    // No move found
                    stopped = true;
                    result = Some(reason);
                    header += &format!(
                        "[Checkmate_time \"{}\"]\n",
                        checkmate_start.elapsed().as_millis()
                    );
                    header += &format!(
                        "[Checkmate_difficulty \"{}\"]\n",
                        checkmate_start.elapsed().as_nanos() as f64 / performance.as_nanos() as f64
                    );
                    match reason {
                        RandomLegalMovesetReason::Error => {
                            println!("Error!");
                        }
                        RandomLegalMovesetReason::TimeoutCheckmate => {
                            if partial_game.info.active_player {
                                header += "[Result \"0-1\"]\n";
                            } else {
                                header += "[Result \"1-0\"]\n";
                            }
                            header += "[Checkmate_timeout \"true\"]\n";
                            println!("Timed out while looking for checkmate!");
                        }
                        RandomLegalMovesetReason::TimeoutStalemate => {
                            header += "[Result \"1/2-1/2\"]\n";
                            header += "[Checkmate_timeout \"true\"]\n";
                            println!("Timed out while looking for stalemate!");
                        }
                        RandomLegalMovesetReason::Checkmate => {
                            if partial_game.info.active_player {
                                header += "[Result \"0-1\"]\n";
                            } else {
                                header += "[Result \"1-0\"]\n";
                            }
                            header += "[Checkmate_timeout \"false\"]\n";
                            println!("Checkmate!");
                        }
                        RandomLegalMovesetReason::Stalemate => {
                            header += "[Result \"1/2-1/2\"]\n";
                            header += "[Checkmate_timeout \"false\"]\n";
                            // println!("{}", header);
                            // println!("{}", res);
                            // println!("{:#?}", partial_game);
                            println!("Stalemate!");
                        }
                    }

                    break;
                }
            }
        }

        if !stopped {
            header += "[Result \"*\"]\n";
            header += "[Checkmate_timeout \"false\"]\n";
            // println!("Complete!");
            // println!("{}\n{}", header, res);
        }

        let white = if partial_game.info.active_player {
            "white"
        } else {
            "black"
        };
        let hash = format!("{:0>8}", rand::thread_rng().gen_range(0..100000000));

        let path = match result {
            Some(RandomLegalMovesetReason::Error) => String::new(),
            Some(RandomLegalMovesetReason::TimeoutCheckmate) => format!(
                "/tmp/db/standard/{}_timeout/{}-{}.5dpgn",
                white,
                hash,
                timestamp.format("%s")
            ),
            Some(RandomLegalMovesetReason::TimeoutStalemate) => format!(
                "/tmp/db/standard/stalemate_timeout/{}-{}.5dpgn",
                hash,
                timestamp.format("%s")
            ),
            Some(RandomLegalMovesetReason::Checkmate) => format!(
                "/tmp/db/standard/{}/{}-{}.5dpgn",
                white,
                hash,
                timestamp.format("%s")
            ),
            Some(RandomLegalMovesetReason::Stalemate) => format!(
                "/tmp/db/standard/stalemate/{}-{}.5dpgn",
                hash,
                timestamp.format("%s")
            ),
            None => format!(
                "/tmp/db/standard/none/{}-{}.5dpgn",
                hash,
                timestamp.format("%s")
            ),
        };

        if let Some(RandomLegalMovesetReason::Error) = result {
            // noop
        } else {
            println!("-> {}", path);
            // println!("@{}:\n\n{}\n{}", path, header, res);
            let mut file = File::create(path)?;
            file.write_all(format!("{}\n{}", header, res).as_bytes())?;
        }
    }
}
