use chess5dlib::parse::*;
use chess5dlib::game::*;
use std::fs::File;
use std::io::prelude::*;

#[test]
pub fn test_parse_simple() {
    let mut file = File::open("tests/standard-empty.json").unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let game = parse(&contents);

    assert!(game.is_some(), "Couldn't parse JSON file tests/standard-empty.json!");
    let game = game.unwrap();

    assert!(game.boards.len() == 1);
    assert!(game.boards.get(&(0, 0)).is_some());
}
