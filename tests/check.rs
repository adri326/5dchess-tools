use chess5dlib::parse::test::read_and_parse;
use chess5dlib::{
    prelude::*,
    check::*,
};

#[test]
fn test_check_standard() {
    let game = read_and_parse("tests/games/standard-empty.json");
    let partial_game = no_partial_game(&game);

    let idle_boards = generate_idle_boards(&game, &partial_game);
    assert!(idle_boards.is_some());
    let idle_boards = idle_boards.unwrap();

    let board = idle_boards.get_board((0, 1));
    assert!(board.is_some());
    let board = board.unwrap();

    assert_eq!(board.pieces, game.get_board((0, 0)).unwrap().pieces);

    let check = is_threatened(&game, &idle_boards);
    assert_eq!(check, Some((false, None)));
}

#[test]
fn test_check_d4() {
    let game = read_and_parse("tests/games/standard-d4.json");
    let partial_game = no_partial_game(&game);

    let idle_boards = generate_idle_boards(&game, &partial_game);
    assert!(idle_boards.is_some());
    let idle_boards = idle_boards.unwrap();

    let board = idle_boards.get_board((0, 2));
    assert!(board.is_some());
    let board = board.unwrap();

    assert_eq!(board.pieces, game.get_board((0, 1)).unwrap().pieces);
    assert_eq!(
        board.en_passant, None,
        "Asserts that generate_idle_boards removes the en_passant flag."
    );

    let check = is_threatened(&game, &idle_boards);
    assert_eq!(check, Some((false, None)));
}

#[test]
fn test_check_true() {
    let game = read_and_parse("tests/games/standard-check.json");
    let partial_game = no_partial_game(&game);
    let idle_boards = generate_idle_boards(&game, &partial_game).unwrap();

    let check = is_threatened(&game, &idle_boards);
    assert!(check.is_some());
    assert!(check.unwrap().0);
}
