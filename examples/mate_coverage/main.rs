use chess5dlib::parse::test::read_and_parse_opt;
use chess5dlib::*;
use rand::{Rng, prelude::SliceRandom};
use std::fs::read_dir;
use std::time::{Duration, Instant};
use std::path::Path;

fn main() {
    nonmates();
    checkmates();
}

fn nonmates() {
    let mut ok = 0;
    let n_games = 100;

    let dir = read_dir(Path::new("./converted-db/standard/none"));
    assert!(dir.is_ok(), "Can't open `./converted-db/standard/none`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok()).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();

    dir.shuffle(&mut rng);

    let games: Vec<(Game, String)> = dir
        .into_iter()
        .take(n_games * 2)
        .filter_map(|entry| {
            if let Some(ext) = entry.path().as_path().extension() {
                if ext == "json" {
                    return read_and_parse_opt(&entry.path().to_str()?)
                        .map(|g| (g, entry.path().to_str().unwrap().to_string()));
                }
            }
            None
        })
        .collect();

    println!("Testing nonmates, {} random games...", n_games);
    let mut sigma = Duration::new(0, 0);
    let mut eta = Duration::new(0, 0);

    for _ in 0..n_games {
        let game = &games[rng.gen_range(0..games.len())];
        let partial_game = no_partial_game(&game.0);
        let start = Instant::now();
        println!("Analyzing game: {} timelines, {} playable boards ...", game.0.info.len_timelines(), partial_game.own_boards(&game.0).count());
        match is_mate(&game.0, &partial_game, Some(Duration::new(10, 0))) {
            Mate::None(_ms) => {
                ok += 1;
                println!("... Game {}, OK! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                sigma += start.elapsed();
                eta += start.elapsed() / game.0.info.len_timelines() as u32;
            },
            Mate::Error => {
                println!("... Game {}, error while looking for mate!", game.1);
            },
            Mate::TimeoutCheckmate | Mate::TimeoutStalemate => {
                println!("... Game {}, timed out while looking for mate!", game.1);
            },
            Mate::Checkmate => {
                println!("... Game {}, found checkmate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
            }
            Mate::Stalemate => {
                println!("... Game {}, found stalemate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
            }
        }
    }
    println!("Ok: {} / {}", ok, n_games);
    println!("Average time taken: {} μs/position", (sigma / ok).as_nanos() as f64 / 1000.0);
    println!("Average time taken: {} μs/position/timeline", (eta / ok).as_nanos() as f64 / 1000.0);
}

fn checkmates() {
    let mut ok = 0;
    let n_games = 100;

    let dir = read_dir(Path::new("./converted-db/standard/black"));
    assert!(dir.is_ok(), "Can't open `./converted-db/standard/black`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok()).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();

    dir.shuffle(&mut rng);

    let games: Vec<(Game, String)> = dir
        .into_iter()
        .take(n_games * 2)
        .filter_map(|entry| {
            if let Some(ext) = entry.path().as_path().extension() {
                if ext == "json" {
                    return read_and_parse_opt(&entry.path().to_str()?)
                        .map(|g| (g, entry.path().to_str().unwrap().to_string()));
                }
            }
            None
        })
        .collect();

    println!("Testing checkmates, {} random games...", n_games);
    let mut sigma = Duration::new(0, 0);
    let mut eta = Duration::new(0, 0);

    for _ in 0..n_games {
        let game = &games[rng.gen_range(0..games.len())];
        let partial_game = no_partial_game(&game.0);
        let start = Instant::now();
        println!("Analyzing game: {} timelines, {} playable boards ...", game.0.info.len_timelines(), partial_game.own_boards(&game.0).count());
        match is_mate(&game.0, &partial_game, Some(Duration::new(10, 0))) {
            Mate::None(ms) => {
                println!("... Game {}, found moveset: {}! ({} μs)", game.1, ms, start.elapsed().as_nanos() as f64 / 1000.0);
            },
            Mate::Error => {
                println!("... Game {}, error while looking for mate!", game.1);
            },
            Mate::TimeoutCheckmate | Mate::TimeoutStalemate => {
                println!("... Game {}, timed out while looking for mate!", game.1);
            },
            Mate::Checkmate => {
                println!("... Game {}, found checkmate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
                ok += 1;
                sigma += start.elapsed();
                eta += start.elapsed() / game.0.info.len_timelines() as u32;
            }
            Mate::Stalemate => {
                println!("... Game {}, found stalemate! ({} μs)", game.1, start.elapsed().as_nanos() as f64 / 1000.0);
            }
        }
    }
    println!("Ok: {} / {}", ok, n_games);
    println!("Average time taken: {} μs/position", (sigma / ok).as_nanos() as f64 / 1000.0);
    println!("Average time taken: {} μs/position/timeline", (eta / ok).as_nanos() as f64 / 1000.0);
}
