use chess5dlib::parse::*;
use chess5dlib::prelude::*;
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
    let moveset = Moveset::new(vec![mv], &game.info).unwrap();
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
