
/**
    Status for checkmate detection.
**/

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mate {
    Checkmate,
    Stalemate,
    Error,
    TimeoutCheckmate,
    TimeoutStalemate,
    None,
}

/**
    Checks whether or not the current position is checkmate, stalemate or none of those.
**/
pub fn is_mate<'a>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    max_duration: Option<Duration>,
) -> Mate {

}
