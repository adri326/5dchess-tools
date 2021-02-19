use chess5dlib::{
    prelude::*,
    check::*,
    gen::*,
};
use chess5dlib::parse::test::{read_and_parse, read_and_parse_opt};
use std::fs::read_dir;
use std::path::Path;
use rand::prelude::SliceRandom;

#[test]
fn test_shift_masks() {
    for width in 1..=MAX_BITBOARD_WIDTH {
        for shift in 0..width {
            let rshift_mask = RSHIFT_MASK[(width - 1) * MAX_BITBOARD_WIDTH + shift];
            let lshift_mask = LSHIFT_MASK[(width - 1) * MAX_BITBOARD_WIDTH + shift];

            assert_eq!(lshift_mask, get_bitboard_mask(width as Physical, shift as Physical));
            assert_eq!(rshift_mask, get_bitboard_mask(width as Physical, -(shift as Physical)));

            assert_eq!(rshift_mask & ((1 << width) - 1), (rshift_mask >> width) & ((1 << width) - 1));
            assert_eq!(rshift_mask & ((1 << width) - 1), (rshift_mask >> (2 * width)) & ((1 << width) - 1));
            assert_eq!(lshift_mask & ((1 << width) - 1), (lshift_mask >> width) & ((1 << width) - 1));
            assert_eq!(lshift_mask & ((1 << width) - 1), (lshift_mask >> (2 * width)) & ((1 << width) - 1));

            for n in 0..shift {
                assert_eq!(rshift_mask & (1 << n), 0);
                assert_eq!(lshift_mask & (1 << (width - n - 1)), 0);
            }
            for n in shift..width {
                assert_eq!(rshift_mask & (1 << n), (1 << n));
                assert_eq!(lshift_mask & (1 << (width - n - 1)), (1 << (width - n - 1)));
            }
        }
    }

    let x: BitBoardPrimitive = 0b0100101000000100;
    assert_eq!(bitboard_shift(x, 1, 0, 4, 4), 0b1000010000001000);
    assert_eq!(bitboard_shift(x, 0, 1, 4, 4), 0b1010000001000000);
    assert_eq!(bitboard_shift(x, -1, 0, 4, 4), 0b0010010100000010);
    assert_eq!(bitboard_shift(x, 0, -1, 4, 4), 0b0000010010100000);
    assert_eq!(bitboard_shift(bitboard_shift(x, 0, 1, 4, 4), 1, 0, 4, 4), bitboard_shift(x, 1, 1, 4, 4));
    assert_eq!(bitboard_shift(bitboard_shift(x, 0, 1, 4, 4), -2, 0, 4, 4), bitboard_shift(x, -2, 1, 4, 4));
    assert_eq!(bitboard_shift(bitboard_shift(x, 2, 0, 4, 4), 0, -1, 4, 4), bitboard_shift(x, 2, -1, 4, 4));
}

#[test]
fn test_pawn_check() {
    let game = read_and_parse("tests/games/bitboard/pawn-check.json");
    let partial_game = no_partial_game(&game);
    let idle = generate_idle_boards(&game, &partial_game).unwrap();

    assert_eq!(is_threatened_bitboard(&game, &idle), Some((true, Move::new(&game, &idle, Coords(0, 9, 5, 1), Coords(0, 9, 4, 0)))));
}

#[test]
fn test_queen_check() {
    let game = read_and_parse("tests/games/bitboard/queen-check.json");
    let partial_game = no_partial_game(&game);
    let idle = generate_idle_boards(&game, &partial_game).unwrap();

    assert_eq!(is_threatened_bitboard(&game, &idle), Some((true, Move::new(&game, &idle, Coords(0, 5, 7, 3), Coords(0, 5, 4, 0)))));
}

#[test]
fn test_rook_check() {
    let game = read_and_parse("tests/games/bitboard/rook-check.json");
    let partial_game = no_partial_game(&game);
    let idle = generate_idle_boards(&game, &partial_game).unwrap();

    assert_eq!(is_threatened_bitboard(&game, &idle), Some((true, Move::new(&game, &idle, Coords(0, 7, 4, 5), Coords(0, 7, 4, 0)))));
}

#[test]
fn test_knight_check() {
    let game = read_and_parse("tests/games/bitboard/knight-check.json");
    let partial_game = no_partial_game(&game);
    let idle = generate_idle_boards(&game, &partial_game).unwrap();

    assert_eq!(is_threatened_bitboard(&game, &idle), Some((true, Move::new(&game, &idle, Coords(0, 7, 5, 2), Coords(0, 7, 4, 0)))));
}

#[test]
fn test_bishop_time_check() {
    let game = read_and_parse("tests/games/bitboard/bishop-timecheck.json");
    let partial_game = no_partial_game(&game);
    let idle = generate_idle_boards(&game, &partial_game).unwrap();

    assert_eq!(is_threatened_bitboard(&game, &idle), Some((true, Move::new(&game, &idle, Coords(0, 11, 4, 5), Coords(0, 1, 4, 0)))));
}

#[test]
fn test_unicorn_time_check() {
    let game = read_and_parse("tests/games/bitboard/unicorn-timecheck.json");
    let partial_game = no_partial_game(&game);
    let idle = generate_idle_boards(&game, &partial_game).unwrap();

    assert_eq!(is_threatened_bitboard(&game, &idle), Some((true, Move::new(&game, &idle, Coords(0, 9, 0, 4), Coords(0, 1, 4, 0)))));
}

#[test]
fn test_unicorn_dimension_check() {
    let game = read_and_parse("tests/games/bitboard/unicorn-dimension-check.json");
    let partial_game = no_partial_game(&game);
    let idle = generate_idle_boards(&game, &partial_game).unwrap();

    assert_eq!(is_threatened_bitboard(&game, &idle), Some((true, Move::new(&game, &idle, Coords(-1, 7, 7, 3), Coords(2, 7, 4, 0)))));
}

#[test]
fn test_random_positions() {
    let dir = read_dir(Path::new("./converted-db/standard/none"));
    assert!(dir.is_ok(), "Can't open `./converted-db/standard/none`");
    let mut dir: Vec<_> = dir.unwrap().filter_map(|entry| entry.ok()).collect();

    let mut rng = rand::thread_rng();

    dir.shuffle(&mut rng);

    for entry in dir.iter().take(1000) {
        if let Some(ext) = entry.path().as_path().extension() {
            if ext == "json" {
                if let Some(game) = read_and_parse_opt(&entry.path().to_str().unwrap()) {
                    test_random_positions_sub(&game, &entry.path().to_str().unwrap());
                }
            }
        }
    }
}

fn test_random_positions_sub(game: &Game, path: &str) {
    let partial_game = no_partial_game(game);
    let idle = generate_idle_boards(game, &partial_game).unwrap();

    match (is_threatened_bitboard(game, &idle), is_threatened(game, &idle)) {
        (Some((true, _)), Some((true, _))) => {
            // Ok
        }
        (Some((false, _)), Some((false, _))) => {
            // Ok
        }
        (Some((true, res)), Some((false, _))) => {
            assert!(false, "is_threatened_bitboard found a move but is_threatened didn't!\n=> {:?}\n@ {}", res, path);
        }
        (Some((false, _)), Some((true, res))) => {
            assert!(false, "is_threatened found a move but is_threatened_bitboard didn't!\n=> {:?}\n@ {}", res, path);
        }
        x => panic!("Error while looking for threatening moves: {:?}\n@ {}", x, path),
    }

    for board in idle.opponent_boards(game) {
        let mut physical_checking_moves: Vec<Move> = Vec::new();

        for mv in board.generate_moves_flag(game, &idle, GenMovesFlag::Check).unwrap() {
            match mv.to.0 {
                Some(piece) => {
                    if piece.is_royal() && piece.white == partial_game.info.active_player && mv.to.1.non_physical() == mv.from.1.non_physical() {
                        physical_checking_moves.push(mv);
                    }
                }
                None => {}
            }
        }

        if let Some((from_x, from_y, to_x, to_y)) = threats_within_board(board) {
            let mv = Move::new(game, &idle, Coords(board.l(), board.t(), from_x, from_y), Coords(board.l(), board.t(), to_x, to_y)).unwrap();
            if physical_checking_moves.iter().find(|x| **x == mv).is_none() {
                assert!(false, "threats_within_board found a threat that wasn't found by generate_moves_flag: {:?}\n@ {}", mv, path);
            }
        } else {
            if physical_checking_moves.len() > 0 {
                assert!(false, "threats_within_board found no threat when generate_moves_flag found some: {:?}\n@ {}", physical_checking_moves[0], path);
            }
        }
    }
}
