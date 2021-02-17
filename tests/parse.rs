use chess5dlib::parse::*;
use chess5dlib::prelude::*;
use std::fs::File;
use std::io::prelude::*;

#[test]
pub fn test_parse_standard() {
    let mut file = File::open("tests/games/standard-empty.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents);

    assert!(
        game.is_some(),
        "Couldn't parse JSON file tests/games/standard-empty.json!"
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

    let bitboards = game.get_board((0, 0)).unwrap().bitboards;
    assert_eq!(bitboards.white_royal, 1u128 << 4);
    assert_eq!(bitboards.black_royal, 1u128 << (7 * 8 + 4));
    assert_eq!(bitboards.white[0], 0x000000000000ff00);
    assert_eq!(bitboards.black[0], 0x00ff000000000000);
    assert_eq!(bitboards.white[1], 0x0000000000000000);
    assert_eq!(bitboards.black[1], 0x0000000000000000);
    assert_eq!(bitboards.white[2], 1u128 << 4);
    assert_eq!(bitboards.black[2], 1u128 << (7 * 8 + 4));
    assert_eq!(bitboards.white[3], 1u128 << 4);
    assert_eq!(bitboards.black[3], 1u128 << (7 * 8 + 4));
    assert_eq!(bitboards.white[4], 1u128 << 4);
    assert_eq!(bitboards.black[4], 1u128 << (7 * 8 + 4));
    assert_eq!(bitboards.white[5], 1u128 << 4);
    assert_eq!(bitboards.black[5], 1u128 << (7 * 8 + 4));
    assert_eq!(bitboards.white[6], 1u128 | (1u128 << 3) | (1u128 << 7));
    assert_eq!(bitboards.black[6], (1u128 | (1u128 << 3) | (1u128 << 7)) << (7 * 8));
    assert_eq!(bitboards.white[7], (1u128 << 2) | (1u128 << 3) | (1u128 << 5));
    assert_eq!(bitboards.black[7], ((1u128 << 2) | (1u128 << 3) | (1u128 << 5)) << (7 * 8));
    assert_eq!(bitboards.white[8], 1u128 << 3);
    assert_eq!(bitboards.black[8], 1u128 << (7 * 8 + 3));
    assert_eq!(bitboards.white[9], 1u128 << 3);
    assert_eq!(bitboards.black[9], 1u128 << (7 * 8 + 3));
    assert_eq!(bitboards.white[10], (1u128 << 1) | (1u128 << 6));
    assert_eq!(bitboards.black[10], ((1u128 << 1) | (1u128 << 6)) << (7 * 8));
}

#[test]
pub fn test_parse_standard_t0() {
    let mut file = File::open("tests/games/standard-t0-empty.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents);

    assert!(
        game.is_some(),
        "Couldn't parse JSON file tests/games/standard-empty.json!"
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
    let mut file = File::open("tests/games/standard-d4.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents);

    assert!(
        game.is_some(),
        "Couldn't parse JSON file tests/games/standard-d4.json!"
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
                assert!(game.get_board((0, 1)).unwrap().en_passant() == Some((3, 2)));
            } else {
                assert!(game
                    .get(Coords(0, 1, x, y))
                    .map(|p| !p.moved)
                    .unwrap_or(true));
            }
        }
    }
}
