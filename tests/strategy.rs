use chess5dlib::parse::test::{read_and_parse, read_and_parse_opt};
use chess5dlib::prelude::*;
use chess5dlib::utils::*;
use rand::Rng;
use scoped_threadpool::Pool;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::read_dir;
use std::hash::Hash;
use std::path::Path;

// I'm very sorry.
// - Shad

fn compare_methods<F1, F2, M>(
    iterations: usize,
    max_playable_boards: usize,
    method_1: F1,
    method_2: F2,
) where
    for<'a> F1: Fn(&'a Game, &'a PartialGame<'a>) -> Vec<M> + Copy + Send + Sync,
    for<'a> F2: Fn(&'a Game, &'a PartialGame<'a>) -> Vec<M> + Copy + Send + Sync,
    M: Eq + Hash + Debug,
{
    let dir = read_dir(Path::new("./converted-db/standard/none"));
    assert!(dir.is_ok(), "Can't open `./converted-db/standard/none`");
    let dir = dir.unwrap().filter_map(|entry| entry.ok());

    let games: Vec<(Game, String)> = dir
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

    assert!(games.len() > 1);

    let mut pool = Pool::new(num_cpus::get() as u32);

    {
        let games = &games;
        pool.scoped(move |scope| {
            for _ in 0..iterations {
                let method_1 = method_1;
                let method_2 = method_2;
                scope.execute(move || {
                    let mut rng = rand::thread_rng();

                    let game = &games[rng.gen_range(0..games.len())];
                    let partial_game: PartialGame = PartialGame::from(&game.0);
                    if partial_game.own_boards(&game.0).count() <= max_playable_boards {
                        let res_1: HashSet<M> =
                            method_1(&game.0, &partial_game).into_iter().collect();
                        let res_2: HashSet<M> =
                            method_2(&game.0, &partial_game).into_iter().collect();

                        if res_1 != res_2 {
                            println!("Missing moves in method_1: {:?}", res_2.difference(&res_1));
                            println!("Missing moves in method_2: {:?}", res_1.difference(&res_2));
                        }

                        assert_eq!(
                            res_1, res_2,
                            "Failed to generate the same moves on game {}",
                            game.1
                        );
                    }
                });
            }
        });
    }
}

#[test]
fn test_compare_self() {
    compare_methods(
        100,
        2,
        |game, partial_game: &PartialGame<'_>| {
            GenMovesetIter::new(partial_game.own_boards(game).collect(), game, partial_game)
                .flatten()
                .collect()
        },
        |game, partial_game: &PartialGame<'_>| {
            GenMovesetIter::new(partial_game.own_boards(game).collect(), game, partial_game)
                .flatten()
                .collect()
        },
    );
}

#[test]
#[should_panic]
fn test_compare_err() {
    compare_methods(
        100,
        2,
        |game, partial_game: &PartialGame<'_>| {
            GenMovesetIter::new(partial_game.own_boards(game).collect(), game, partial_game)
                .flatten()
                .collect()
        },
        |_game, _partial_game| vec![],
    );
}

#[test]
fn test_legal_move() {
    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>,
                         game: &Game,
                         partial_game: &PartialGame| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(game, partial_game)?;
            if is_illegal(game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    compare_methods(
        250,
        2,
        |game, partial_game| {
            GenMovesetIter::new(partial_game.own_boards(game).collect(), game, partial_game)
                .flatten()
                .filter_map(|ms| filter_lambda(ms, game, partial_game))
                .collect()
        },
        |game, partial_game| {
            generate_movesets_prefilter(
                partial_game.own_boards(game).collect(),
                game,
                partial_game,
            )
            .flatten()
            .filter_map(|ms| filter_lambda(ms, game, partial_game))
            .collect()
        },
    );
}

#[test]
fn defended_pawn_checkmate() {
    let game = read_and_parse("tests/games/defended-pawn-checkmate.json");
    let partial_game = no_partial_game(&game);

    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
            if is_illegal(&game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    let mut iter = generate_movesets_prefilter(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| filter_lambda(ms));

    let mv = iter.next();

    assert!(
        mv.is_none(),
        "Expected no legal movesets to be found; found {:?}",
        mv
    );
}

#[test]
fn standard_checkmate() {
    let game = read_and_parse("tests/games/standard-checkmate.json");
    let partial_game = no_partial_game(&game);

    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
            if is_illegal(&game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    let mut iter = generate_movesets_prefilter(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| filter_lambda(ms));

    let mv = iter.next();

    assert!(
        mv.is_none(),
        "Expected no legal movesets to be found; found {:?}",
        mv
    );
}

#[test]
fn standard_checkmate_2() {
    let game = read_and_parse("tests/games/standard-checkmate-2.json");
    let partial_game = no_partial_game(&game);

    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
            if is_illegal(&game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    let mut iter = generate_movesets_prefilter(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| filter_lambda(ms));

    let mv = iter.next();

    assert!(
        mv.is_none(),
        "Expected no legal movesets to be found; found {:?}",
        mv
    );
}

#[test]
fn standard_checkmate_3() {
    let game = read_and_parse("tests/games/standard-checkmate-3.json");
    let partial_game = no_partial_game(&game);

    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
            if is_illegal(&game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    let mut iter = generate_movesets_prefilter(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| filter_lambda(ms));

    let mv = iter.next();

    assert!(
        mv.is_none(),
        "Expected no legal movesets to be found; found {:?}",
        mv
    );
}

#[test]
fn princess_checkmate() {
    let game = read_and_parse("tests/games/princess-checkmate.json");
    let partial_game = no_partial_game(&game);

    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
            if is_illegal(&game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    let mut iter = generate_movesets_prefilter(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| filter_lambda(ms));

    let mv = iter.next();

    if let Some(ms) = mv.clone() {
        let new_partial_game = ms.generate_partial_game(&game, &partial_game).unwrap();
        println!("{:#?}", new_partial_game);
        for board in new_partial_game.own_boards(&game) {
            println!("{:?}", board);
        }
    }

    assert!(
        mv.is_none(),
        "Expected no legal movesets to be found; found {:?}",
        mv
    );
}

#[test]
fn tricky_nonmate() {
    let game = read_and_parse("tests/games/tricky-nonmate.json");
    let partial_game = no_partial_game(&game);

    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
            if is_illegal(&game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    let mut iter = generate_movesets_prefilter(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| filter_lambda(ms));

    let mv = iter.next();

    assert!(
        mv.is_some(),
        "Expected a legal movesets to be found; found none."
    );
}

#[test]
fn reflected_checkmate() {
    let game = read_and_parse("tests/games/reflected-checkmate.json");
    let partial_game = no_partial_game(&game);

    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
            if is_illegal(&game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    let mut iter = generate_movesets_prefilter(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| filter_lambda(ms));

    let mv = iter.next();

    assert!(
        mv.is_none(),
        "Expected no legal movesets to be found; found {:?}",
        mv
    );
}

#[test]
fn standard_nonmate() {
    let game = read_and_parse("tests/games/standard-nonmate.json");
    let partial_game = no_partial_game(&game);

    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
            if is_illegal(&game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    let mut iter = generate_movesets_prefilter(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| filter_lambda(ms));

    let mv = iter.next();

    assert!(
        mv.is_some(),
        "Expected a legal moveset to be found; found None"
    );

    let mv = random_legal_moveset(&game, &partial_game, None);
    assert!(
        mv.is_ok(),
        "Expected a legal moveset to be found; found {:?}",
        mv
    );
}

#[test]
fn standard_nonmate2() {
    let game = read_and_parse("tests/games/standard-nonmate-2.json");
    let partial_game = no_partial_game(&game);

    // let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>| match ms {
    //     Ok(ms) => {
    //         let new_partial_game = ms.generate_partial_game(&game, &partial_game)?;
    //         if is_illegal(&game, &new_partial_game)? {
    //             None
    //         } else {
    //             Some(ms)
    //         }
    //     }
    //     Err(_) => None,
    // };

    // let mut iter = generate_movesets_prefilter(
    //     partial_game.own_boards(&game).collect(),
    //     &game,
    //     &partial_game,
    //     LegalMove::new(),
    // )
    // .flatten()
    // .filter_map(|ms| filter_lambda(ms));

    // let mv = iter.next();

    // assert!(
    //     mv.is_some(),
    //     "Expected a legal moveset to be found; found None"
    // );

    let mv = random_legal_moveset(&game, &partial_game, Some(std::time::Duration::new(5, 0)));
    assert!(
        mv.is_ok(),
        "Expected a legal moveset to be found; found {:?}",
        mv
    );
}

#[test]
fn test_list_legal_movesets() {
    let filter_lambda = |ms: Result<Moveset, MovesetValidityErr>,
                         game: &Game,
                         partial_game: &PartialGame| match ms {
        Ok(ms) => {
            let new_partial_game = ms.generate_partial_game(game, partial_game)?;
            if is_illegal(game, &new_partial_game)? {
                None
            } else {
                Some(ms)
            }
        }
        Err(_) => None,
    };

    compare_methods(
        10,
        3,
        |game, partial_game| {
            generate_movesets_prefilter(
                partial_game.own_boards(game).collect(),
                game,
                partial_game,
            )
            .flatten()
            .filter_map(|ms| filter_lambda(ms, game, partial_game))
            .collect()
        },
        |game, partial_game| {
            list_legal_movesets(game, partial_game, None)
                .map(|(mv, _ms)| mv)
                .collect()
        },
    );
}
