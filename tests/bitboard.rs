use chess5dlib::prelude::*;
use chess5dlib::parse::test::read_and_parse;

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
