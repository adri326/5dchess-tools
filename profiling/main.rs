#[allow(unused_imports)]
use chess5dlib::{parse::*, prelude::*, utils::*};
use std::fs::File;
use std::fs::read_dir;
use std::io::prelude::*;
use std::time::{Instant, Duration};
use rand::Rng;
use scoped_threadpool::Pool;

extern crate coz;

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();

    let (duration, threads) = if args.len() != 3 {
        println!("No duration or thread count specified, exitting!");
        return Ok(())
    } else {
        match (args[1].parse(), args[2].parse()) {
            (Ok(x), Ok(y)) => (Duration::new(x, 0), y),
            _ => {
                println!("Invalid duration or thread count, exitting!");
                return Ok(())
            }
        }
    };

    let files = read_dir("./converted-db/nonmate")?;
    let mut games: Vec<Game> = Vec::new();

    for path in files {
        let mut file = File::open(path?.path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        games.push(parse(&contents).unwrap());
    }

    let started = Instant::now();

    let mut pool = Pool::new(threads);

    {
        let games = &games;

        pool.scoped(move |scope| {
            for n in 0..threads {
                scope.execute(move || {
                    let mut rng = rand::thread_rng();
                    let mut sigma = 0;
                    let mut delta = Duration::new(0, 0);
                    let mut epsilon = 0;
                    while started.elapsed() < duration {
                        let game = &games[rng.gen_range(0..games.len())];
                        let partial_game = no_partial_game(game);

                        let (dsigma, ddelta) = test(started, duration, game, &partial_game, 1);
                        if started.elapsed() < duration {
                            epsilon += 1;
                        }
                        sigma += dsigma;
                        delta += ddelta;
                    }
                    println!("Thread {}: {:?} moveset/ms", n, sigma as f64 / delta.as_millis() as f64);
                    // println!("Thread {}: {:?} position/s", n, epsilon as f64 / delta.as_millis() as f64 * 1000.0);
                });
            }
        });
    }

    Ok(())
}

fn test(started: Instant, duration: Duration, game: &Game, partial_game: &PartialGame<Board>, _iterations: usize) -> (usize, Duration) {
    let mut sigma = 0;
    let mut delta = Duration::new(0, 0);

    coz::begin!("new");
    let mut iter = list_legal_movesets(game, partial_game, Some(Duration::new(30, 0)));
    coz::end!("new");

    loop {
        let begin = Instant::now();
        coz::begin!("next");
        let next = iter.next();
        coz::end!("next");
        sigma += 1;
        delta += begin.elapsed();

        if next.is_none() {
            break
        }
        if started.elapsed() > duration {
            break
        }
    }

    (sigma, delta)
}
