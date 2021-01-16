use chess5dlib::prelude::*;
use chess5dlib::parse::parse;
use std::fs::File;
use std::io::Read;

pub fn read_and_parse(path: String) -> Option<Game> {
    let mut file = File::open(&path).ok()?;
    let mut contents = String::new();

    file.read_to_string(&mut contents).ok()?;

    parse(&contents)
}

#[test]
fn test_own_boards() {
    let game = read_and_parse(String::from("tests/games/standard-empty.json")).unwrap();
    let partial_game = no_partial_game(&game);
    let own_boards: Vec<&Board> = partial_game.own_boards(&game).collect();

    assert_eq!(own_boards, vec![game.get_board((0, 0)).unwrap()]);

    let game = read_and_parse(String::from("tests/games/standard-d4.json")).unwrap();
    let partial_game = no_partial_game(&game);
    let own_boards: Vec<&Board> = partial_game.own_boards(&game).collect();

    assert_eq!(own_boards, vec![game.get_board((0, 1)).unwrap()]);
}
