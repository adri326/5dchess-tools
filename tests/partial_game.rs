use chess5dlib::parse::test::read_and_parse;
use chess5dlib::prelude::*;

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
