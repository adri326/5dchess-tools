use chess5dlib::parse::*;
use chess5dlib::prelude::*;
use std::fs::File;
use std::io::prelude::*;

#[test]
pub fn test_parse_standard() {
    let mut file = File::open("tests/standard-empty.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents);

    assert!(
        game.is_some(),
        "Couldn't parse JSON file tests/standard-empty.json!"
    );
    let game = game.unwrap();

    assert!(game.boards.len() == 1);
    assert!(game.boards.get(&(0, 0)).is_some());

    // Check that every piece on the starting board hasn't moved
    for x in 0..(8 as Physical) {
        for y in 0..(8 as Physical) {
            assert!(game
                .get(Coords(0, 0, x, y))
                .map(|p| !p.moved)
                .unwrap_or(true));
        }
    }
}

#[test]
pub fn test_parse_standard_t0() {
    let mut file = File::open("tests/standard-t0-empty.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents);

    assert!(
        game.is_some(),
        "Couldn't parse JSON file tests/standard-empty.json!"
    );
    let game = game.unwrap();

    assert_eq!(game.boards.len(), 2);
    assert!(game.boards.get(&(0, 0)).is_some());

    // Check that every piece on the starting board hasn't moved
    for x in 0..(8 as Physical) {
        for y in 0..(8 as Physical) {
            assert!(game
                .get(Coords(0, -1, x, y))
                .map(|p| !p.moved)
                .unwrap_or(true));
            assert!(game
                .get(Coords(0, 0, x, y))
                .map(|p| !p.moved)
                .unwrap_or(true));
        }
    }
}
#[test]
pub fn test_parse_standard_d4() {
    let mut file = File::open("tests/standard-d4.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents);

    assert!(
        game.is_some(),
        "Couldn't parse JSON file tests/standard-d4.json!"
    );
    let game = game.unwrap();

    assert!(game.boards.len() == 2);
    assert!(game.boards.get(&(0, 0)).is_some());
    assert!(game.boards.get(&(0, 1)).is_some());

    // Check that every piece on the starting board hasn't moved
    for x in 0..(8 as Physical) {
        for y in 0..(8 as Physical) {
            assert!(game
                .get(Coords(0, 0, x, y))
                .map(|p| !p.moved)
                .unwrap_or(true));
        }
    }

    for x in 0..(8 as Physical) {
        for y in 0..(8 as Physical) {
            if x == 3 && y == 1 {
                assert_eq!(game.get(Coords(0, 1, x, y)), Tile::Blank);
            } else if x == 3 && y == 3 {
                assert!(game.get(Coords(0, 1, x, y)).piece().is_some());
                assert!(game.get_board((0, 1)).unwrap().en_passant == Some((3, 2)));
            } else {
                assert!(game
                    .get(Coords(0, 1, x, y))
                    .map(|p| !p.moved)
                    .unwrap_or(true));
            }
        }
    }
}
