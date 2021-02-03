use chess5dlib::parse::test::read_and_parse;
use chess5dlib::prelude::*;
use chess5dlib::strategies::misc::NoCastling;
use std::convert::TryFrom;

#[test]
fn test_own_boards() {
    let game = read_and_parse("tests/games/standard-empty.json");
    let partial_game = no_partial_game(&game);
    let own_boards: Vec<&Board> = partial_game.own_boards(&game).collect();

    assert_eq!(own_boards, vec![game.get_board((0, 0)).unwrap()]);

    let game = read_and_parse("tests/games/standard-d4.json");
    let partial_game = no_partial_game(&game);
    let own_boards: Vec<&Board> = partial_game.own_boards(&game).collect();

    assert_eq!(own_boards, vec![game.get_board((0, 1)).unwrap()]);
}

#[test]
fn test_new_partial_game_d4() {
    // Tests that playing d4 yields the right state
    let game_empty = read_and_parse("tests/games/standard-empty.json");
    let partial_game_empty = no_partial_game(&game_empty);
    let game_d4 = read_and_parse("tests/games/standard-d4.json");
    let partial_game_d4 = no_partial_game(&game_d4);

    // d2-d4
    let mv = Move::new(
        &game_empty,
        &partial_game_empty,
        Coords(0, 0, 3, 1),
        Coords(0, 0, 3, 3),
    )
    .unwrap();
    let ms = Moveset::try_from((vec![mv], &game_empty.info)).unwrap();

    let new_partial_game = ms
        .generate_partial_game(&game_empty, &partial_game_empty)
        .unwrap();

    assert_eq!(new_partial_game.info, game_d4.info);
    assert_eq!(
        new_partial_game
            .own_boards(&game_empty)
            .collect::<Vec<&Board>>(),
        partial_game_d4
            .own_boards(&game_d4)
            .collect::<Vec<&Board>>()
    );
}

#[test]
fn test_new_partial_game_castling() {
    // Tests that every move from a position where castling is legal, thus including the move where white castles

    let game = read_and_parse("tests/games/standard-castle.json");
    let partial_game = no_partial_game(&game);

    let movesets: Vec<Moveset> = GenMovesetIter::new(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
    )
    .flatten()
    .filter_map(|ms| ms.ok())
    .collect();

    assert!(
        movesets
            .iter()
            .find(|ms| ms.moves()[0].kind == MoveKind::Castle)
            .is_some(),
        "∃ a castling move"
    );

    for ms in movesets {
        let new_partial_game = ms.generate_partial_game(&game, &partial_game);
        assert!(
            new_partial_game.is_some(),
            "Expected a partial game to be generated!"
        );
        let new_partial_game = new_partial_game.unwrap();
        assert!(
            new_partial_game.info.present <= partial_game.info.present + 1,
            "Left: {} <= Right: {}? --- MS: {:?};\n=> After: {:#?}\n=> Before: {:#?}",
            new_partial_game.info.present,
            partial_game.info.present + 1,
            ms,
            new_partial_game.info,
            partial_game.info,
        );
    }

    let movesets_no_castling: Vec<Moveset> = generate_movesets_filter_strategy::<NoCastling>(
        partial_game.own_boards(&game).collect(),
        &game,
        &partial_game,
        NoCastling::new(),
    )
    .flatten()
    .filter_map(|ms| ms.ok())
    .collect();

    assert!(
        movesets_no_castling
            .iter()
            .find(|ms| ms.moves()[0].kind == MoveKind::Castle)
            .is_none(),
        "∄ a castling move when castling is filtered out"
    );
}
