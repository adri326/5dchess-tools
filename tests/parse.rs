use chess5dlib::prelude::*;
use chess5dlib::parse::*;
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
    for x in 0..8u8 {
        for y in 0..8u8 {
            assert!(game.get(Coords(0, 0, x, y)).map(|p| !p.moved).unwrap_or(true));
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
    for x in 0..8u8 {
        for y in 0..8u8 {
            assert!(game.get(Coords(0, -1, x, y)).map(|p| !p.moved).unwrap_or(true));
            assert!(game.get(Coords(0, 0, x, y)).map(|p| !p.moved).unwrap_or(true));
        }
    }
}
