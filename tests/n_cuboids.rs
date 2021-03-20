use chess5dlib::parse::test::{read_and_parse};
#[allow(unused_imports)]
use chess5dlib::{
    prelude::*,
    random::*,
    check::*,
    gen::*,
    gen::n_cuboids::*,
    mate::*,
};
use std::time::Duration;

#[test]
fn standard_checkmate() {
    let game = read_and_parse("tests/games/standard-checkmate.json");
    let partial_game = no_partial_game(&game);

    assert_eq!(is_mate(&game, &partial_game, Some(Duration::new(10, 0))), Mate::Checkmate);
}


#[test]
fn standard_checkmate_2() {
    let game = read_and_parse("tests/games/standard-checkmate-2.json");
    let partial_game = no_partial_game(&game);

    assert_eq!(is_mate(&game, &partial_game, Some(Duration::new(10, 0))), Mate::Checkmate);
}
