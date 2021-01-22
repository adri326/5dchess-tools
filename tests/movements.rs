use chess5dlib::parse::test::read_and_parse;
use chess5dlib::prelude::*;
use std::collections::HashSet;

#[test]
pub fn test_standard_nc3() {
    let game = read_and_parse("tests/games/standard-empty.json");

    let mv = Move::new(
        &game,
        &no_partial_game(&game),
        Coords::new(0, 0, 1, 0),
        Coords::new(0, 0, 2, 2),
    )
    .unwrap();
    let _moveset = Moveset::new(vec![mv], &game.info).unwrap();
}

#[test]
pub fn test_standard_invalid_move() {
    let game = read_and_parse("tests/games/standard-empty.json");

    let mv = Move::new(
        &game,
        &no_partial_game(&game),
        Coords::new(0, 0, 1, 0),
        Coords::new(0, 2, 2, 2),
    )
    .unwrap();
    assert!(Moveset::new(vec![mv], &game.info).is_err());
}

#[test]
pub fn test_standard_empty_moves() {
    let game = read_and_parse("tests/games/standard-empty.json");

    test_piece_movement(
        &game,
        &no_partial_game(&game),
        Coords::new(0, 0, 1, 0),
        vec![Coords::new(0, 0, 0, 2), Coords::new(0, 0, 2, 2)],
    );

    test_piece_movement(
        &game,
        &no_partial_game(&game),
        Coords::new(0, 0, 4, 1),
        vec![Coords::new(0, 0, 4, 2), Coords::new(0, 0, 4, 3)],
    );
}

#[test]
pub fn test_standard_d4d5_moves() {
    let game = read_and_parse("tests/games/standard-d4d5.json");

    // c1-bishop
    test_piece_movement(
        &game,
        &no_partial_game(&game),
        Coords::new(0, 2, 2, 0),
        vec![
            Coords::new(0, 2, 3, 1),
            Coords::new(0, 2, 4, 2),
            Coords::new(0, 2, 5, 3),
            Coords::new(0, 2, 6, 4),
            Coords::new(0, 2, 7, 5),
        ],
    );

    // e1-king
    test_piece_movement(
        &game,
        &no_partial_game(&game),
        Coords::new(0, 2, 4, 0),
        vec![Coords::new(0, 2, 3, 1)],
    );

    // b1-knight
    test_piece_movement(
        &game,
        &no_partial_game(&game),
        Coords(0, 2, 1, 0),
        vec![
            Coords(0, 2, 2, 2),
            Coords(0, 2, 0, 2),
            Coords(0, 0, 1, 2),
            Coords(0, 2, 3, 1),
        ],
    );
}

#[test]
pub fn test_standard_king_moves() {
    let game = read_and_parse("tests/games/standard-castle.json");

    // e1-king
    test_piece_movement(
        &game,
        &no_partial_game(&game),
        Coords::new(0, 6, 4, 0),
        vec![
            Coords::new(0, 6, 5, 0),
            Coords::new(0, 6, 6, 0),
            Coords::new(0, 4, 4, 1),
        ],
    );
}

pub fn test_piece_movement<'a, B: Clone + AsRef<Board>>(
    game: &Game,
    partial_game: &PartialGame<'a, B>,
    src: Coords,
    targets: Vec<Coords>,
) {
    let piece = PiecePosition::new(game.get(src).piece().unwrap(), src);

    let movements: HashSet<Move> = piece.generate_moves(&game, partial_game).unwrap().collect();
    let mut movements_ground_truth: HashSet<Move> = HashSet::new();

    for target in targets.into_iter() {
        movements_ground_truth
            .insert(Move::new(&game, &no_partial_game(&game), src, target).unwrap());
    }

    assert_eq!(movements, movements_ground_truth);
}
