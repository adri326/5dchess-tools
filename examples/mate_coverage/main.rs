use chess5dlib::parse::test::read_and_parse_opt;
use chess5dlib::{
    prelude::*,
    gen::*,
    mate::*,
};
use rand::{Rng, prelude::SliceRandom};
use std::fs::read_dir;
use std::time::{Duration, Instant};
use std::path::Path;
use std::borrow::Cow;

// Some of the games in the database can be quite wild, so this is to limit the number of total timelines that there are.
// The best case GenLegalMovesetIter's optimizations would be with between 3 and 5 playable boards and complex, inter-timeline checking scenarios
const MAX_TIMELINES: usize = 8;

// bit 0 (2⁰=1): is_mate
// bit 1 (2¹=2): GenLegalMovesetIter
// This means that 7 runs all of the methods and compares their results and 2 only runs GenLegalMovesetIter
const METHOD: u8 = 3;

// The higher, the more games will be analyzed. Games are randomly sampled from the database without putting them back in the pool, so it will take a lot of games to get a statistically representative number.
const N_GAMES: usize = 100;

// Whether or not the program should not report data for each game analyzed
const SILENT: bool = true;

// The maximum number of seconds that each method may take. The higher, the slower (obviously), but the more valid results may come in.
// A value between 3 and 10 seconds should include more than 90% of the games (complexity grows exponentially anyways)
const MAX_SECONDS: u64 = 10;

fn main() {
    checkmates();
    // nonmates();
}

// Note: sigma measures the sum of the time taken to prove mate/nonmate, but only for the valid samples
// eta measures the sum of the time taken to prove mate/nonmate, divided by the number of timelines that that game has and only for the valid samples
// ok measures the number of valid samples

fn nonmates() {
    let dir = read_dir(Path::new("./converted-db/standard/none"));
    assert!(dir.is_ok(), "Can't open `./converted-db/standard/none`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok()).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();

    dir.shuffle(&mut rng);

    let games: Vec<(Game, String)> = dir
        .into_iter()
        .filter_map(|entry| {
            if let Some(ext) = entry.path().as_path().extension() {
                if ext == "json" {
                    match read_and_parse_opt(&entry.path().to_str()?) {
                        Some(game) => {
                            if game.info.len_timelines() <= MAX_TIMELINES {
                                return Some((game, entry.path().to_str().unwrap().to_string()))
                            }
                        },
                        None => {}
                    }
                }
            }
            None
        })
        .take(N_GAMES * 2)
        .collect();

    println!("Testing nonmates, {} random games...", N_GAMES);
    let mut ok = 0;
    let mut sigma = Duration::new(0, 0);
    let mut eta = Duration::new(0, 0);

    let mut ok2 = 0;
    let mut sigma2 = Duration::new(0, 0);
    let mut eta2 = Duration::new(0, 0);

    for _ in 0..N_GAMES {
        let game = &games[rng.gen_range(0..games.len())];
        let partial_game = no_partial_game(&game.0);
        let start = Instant::now();
        if !SILENT {
            println!("Analyzing game: {} timelines, {} playable boards ...", game.0.info.len_timelines(), partial_game.own_boards(&game.0).count());
        }
        if METHOD & 1 > 0 {
            match is_mate(&game.0, &partial_game, Some(Duration::new(MAX_SECONDS, 0))) {
                Mate::None(_ms) => {
                    ok += 1;
                    if !SILENT {
                        println!("... Game {}, OK! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                    }
                    sigma += start.elapsed();
                    eta += start.elapsed() / game.0.info.len_timelines() as u32;
                },
                Mate::Error => {
                    if !SILENT {
                        println!("... Game {}, error while looking for mate!", game.1);
                    }
                },
                Mate::TimeoutCheckmate | Mate::TimeoutStalemate => {
                    if !SILENT {
                        println!("... Game {}, timed out while looking for mate!", game.1);
                    }
                },
                Mate::Checkmate => {
                    if !SILENT {
                        println!("... Game {}, found checkmate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                    }
                }
                Mate::Stalemate => {
                    if !SILENT {
                        println!("... Game {}, found stalemate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                    }
                }
            }
        }
        if METHOD & 2 > 0 {
            let mut iter = GenLegalMovesetIter::new(&game.0, Cow::Borrowed(&partial_game), Some(Duration::new(MAX_SECONDS, 0)));
            match iter.next() {
                Some((_ms, _pos)) => {
                    ok2 += 1;
                    if !SILENT {
                        println!("... Game {}, OK! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                    }
                    sigma2 += start.elapsed();
                    eta2 += start.elapsed() / game.0.info.len_timelines() as u32;
                }
                None => {
                    if iter.timed_out() {
                        if !SILENT {
                            println!("... Game {}, timed out while looking for mate!", game.1);
                        }
                    } else {
                        if !SILENT {
                            println!("... Game {}, found mate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                        }
                    }
                }
            }
        }
    }
    if METHOD & 1 > 0 {
        println!("is_mate() on non-mates:");
        println!("Ok: {} / {}", ok, N_GAMES);
        println!("Average time taken: {} μs/position", (sigma / ok).as_nanos() as f64 / 1000.0);
        println!("Average time taken: {} μs/position/timeline", (eta / ok).as_nanos() as f64 / 1000.0);
    }
    if METHOD & 2 > 0 {
        println!("GenLegalMovesetIter on non-mates:");
        println!("Ok: {} / {}", ok2, N_GAMES);
        println!("Average time taken: {} μs/position", (sigma2 / ok2).as_nanos() as f64 / 1000.0);
        println!("Average time taken: {} μs/position/timeline", (eta2 / ok2).as_nanos() as f64 / 1000.0);
    }
    if METHOD & 3 == 3 {
        println!("GenLegalMovesetIter / is_mate() on non-mates:");
        println!("Average time taken ratio: {}", (sigma / ok).as_nanos() as f64 / (sigma2 / ok2).as_nanos() as f64);
        println!("Average time taken per timeline ratio: {}", (eta / ok).as_nanos() as f64 / (eta2 / ok2).as_nanos() as f64);
    }
}

fn checkmates() {
    let dir = read_dir(Path::new("./converted-db/standard/black"));
    assert!(dir.is_ok(), "Can't open `./converted-db/standard/black`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok()).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();

    dir.shuffle(&mut rng);

    let games: Vec<(Game, String)> = dir
        .into_iter()
        .filter_map(|entry| {
            if let Some(ext) = entry.path().as_path().extension() {
                if ext == "json" {
                    match read_and_parse_opt(&entry.path().to_str()?) {
                        Some(game) => {
                            if game.info.len_timelines() <= MAX_TIMELINES {
                                return Some((game, entry.path().to_str().unwrap().to_string()))
                            }
                        },
                        None => {}
                    }
                }
            }
            None
        })
        .take(N_GAMES * 2)
        .collect();

    println!("Testing checkmates, {} random games...", N_GAMES);
    let mut ok = 0;
    let mut sigma = Duration::new(0, 0);
    let mut eta = Duration::new(0, 0);

    let mut ok2 = 0;
    let mut sigma2 = Duration::new(0, 0);
    let mut eta2 = Duration::new(0, 0);

    for _ in 0..N_GAMES {
        let game = &games[rng.gen_range(0..games.len())];
        let partial_game = no_partial_game(&game.0);
        let start = Instant::now();

        if !SILENT {
            println!("Analyzing game: {} timelines, {} playable boards ...", game.0.info.len_timelines(), partial_game.own_boards(&game.0).count());
        }

        if METHOD & 1 > 0 {
            match is_mate(&game.0, &partial_game, Some(Duration::new(MAX_SECONDS, 0))) {
                Mate::None(ms) => {
                    if !SILENT {
                        println!("... Game {}, found moveset: {}! ({} μs)", game.1, ms, start.elapsed().as_nanos() as f64 / 1000.0);
                    }
                },
                Mate::Error => {
                    if !SILENT {
                        println!("... Game {}, error while looking for mate!", game.1);
                    }
                },
                Mate::TimeoutCheckmate | Mate::TimeoutStalemate => {
                    if !SILENT {
                        println!("... Game {}, timed out while looking for mate!", game.1);
                    }
                },
                Mate::Checkmate => {
                    if !SILENT {
                        println!("... Game {}, found checkmate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                    }
                    ok += 1;
                    sigma += start.elapsed();
                    eta += start.elapsed() / game.0.info.len_timelines() as u32;
                }
                Mate::Stalemate => {
                    if !SILENT {
                        println!("... Game {}, found stalemate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                    }
                }
            }
        }
        if METHOD & 2 > 0 {
            let mut iter = GenLegalMovesetIter::new(&game.0, Cow::Borrowed(&partial_game), Some(Duration::new(MAX_SECONDS, 0)));
            match iter.next() {
                Some((_ms, _pos)) => {
                    if !SILENT {
                        println!("... Game {}, OK! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                    }
                }
                None => {
                    if iter.timed_out() {
                        if !SILENT {
                            println!("... Game {}, timed out while looking for mate!", game.1);
                        }
                    } else {
                        if !SILENT {
                            println!("... Game {}, found mate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                        }
                        ok2 += 1;
                        sigma2 += start.elapsed();
                        eta2 += start.elapsed() / game.0.info.len_timelines() as u32;
                    }
                }
            }
        }
    }

    if METHOD & 1 > 0 {
        println!("is_mate() on checkmates:");
        println!("Ok: {} / {}", ok, N_GAMES);
        println!("Average time taken: {} μs/position", (sigma / ok).as_nanos() as f64 / 1000.0);
        println!("Average time taken: {} μs/position/timeline", (eta / ok).as_nanos() as f64 / 1000.0);
    }

    if METHOD & 2 > 0 {
        println!("GenLegalMovesetIter on checkmates:");
        println!("Ok: {} / {}", ok2, N_GAMES);
        println!("Average time taken: {} μs/position", (sigma2 / ok2).as_nanos() as f64 / 1000.0);
        println!("Average time taken: {} μs/position/timeline", (eta2 / ok2).as_nanos() as f64 / 1000.0);
    }
    if METHOD & 3 == 3 {
        println!("GenLegalMovesetIter / is_mate() on checkmates:");
        println!("Average time taken ratio: {}", (sigma / ok).as_nanos() as f64 / (sigma2 / ok2).as_nanos() as f64);
        println!("Average time taken per timeline ratio: {}", (eta / ok).as_nanos() as f64 / (eta2 / ok2).as_nanos() as f64);
    }
}
