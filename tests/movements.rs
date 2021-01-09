use chess5dlib::parse::*;
use chess5dlib::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

#[test]
pub fn test_standard_nc3() {
    let mut file = File::open("tests/standard-empty.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents).unwrap();

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
    let mut file = File::open("tests/standard-empty.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents).unwrap();

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
    let mut file = File::open("tests/standard-empty.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents).unwrap();

    {
        let position = Coords(0, 0, 1, 0);
        let piece = PiecePosition::new(game.get(position).piece().unwrap(), position);

        let movements: HashSet<Move> = piece
            .generate_moves(&game, &no_partial_game(&game))
            .unwrap()
            .collect();
        let mut movements_ground_truth: HashSet<Move> = HashSet::new();
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 0, 1, 0),
                Coords::new(0, 0, 0, 2),
            )
            .unwrap(),
        );
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 0, 1, 0),
                Coords::new(0, 0, 2, 2),
            )
            .unwrap(),
        );

        assert_eq!(movements, movements_ground_truth);
    }

    {
        let position = Coords(0, 0, 4, 1);
        let piece = PiecePosition::new(game.get(position).piece().unwrap(), position);

        let movements: HashSet<Move> = piece
            .generate_moves(&game, &no_partial_game(&game))
            .unwrap()
            .collect();
        let mut movements_ground_truth: HashSet<Move> = HashSet::new();
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 0, 4, 1),
                Coords::new(0, 0, 4, 2),
            )
            .unwrap(),
        );
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 0, 4, 1),
                Coords::new(0, 0, 4, 3),
            )
            .unwrap(),
        );

        assert_eq!(movements, movements_ground_truth);
    }
}

#[test]
pub fn test_standard_d4d5_moves() {
    let mut file = File::open("tests/standard-d4d5.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents).unwrap();

    {
        let position = Coords(0, 2, 2, 0);
        let piece = PiecePosition::new(game.get(position).piece().unwrap(), position);

        let movements: HashSet<Move> = piece
            .generate_moves(&game, &no_partial_game(&game))
            .unwrap()
            .collect();
        let mut movements_ground_truth: HashSet<Move> = HashSet::new();
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 2, 2, 0),
                Coords::new(0, 2, 3, 1),
            )
            .unwrap(),
        );
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 2, 2, 0),
                Coords::new(0, 2, 4, 2),
            )
            .unwrap(),
        );
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 2, 2, 0),
                Coords::new(0, 2, 5, 3),
            )
            .unwrap(),
        );
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 2, 2, 0),
                Coords::new(0, 2, 6, 4),
            )
            .unwrap(),
        );
        movements_ground_truth.insert(
            Move::new(
                &game,
                &no_partial_game(&game),
                Coords::new(0, 2, 2, 0),
                Coords::new(0, 2, 7, 5),
            )
            .unwrap(),
        );

        assert_eq!(movements, movements_ground_truth);
    }
}
