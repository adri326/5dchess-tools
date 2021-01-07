use super::Piece;

/** Hold the state of a coordinate's pointed tile. Such a tile may either be:
    - a piece (`Tile::Piece(...)`)
    - a blank on a board (`Tile::Blank`)
    - the void (where there is no board or out of bounds) (`Tile::Void`)

    Most pieces cannot cross the void, which is why that distinction is necessary.
**/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Piece(Piece),
    Blank,
    Void
}

impl Tile {
    /** Creates a new `Tile` from an `Option<Piece>`; if `None`, then it will be substituted with `Tile::Blank` **/
    pub fn new_blank(piece_raw: Option<Piece>) -> Tile {
        match piece_raw {
            Some(piece) => Tile::Piece(piece),
            _ => Tile::Blank,
        }
    }

    /** Creates a new `Tile` from an `Option<Piece>`; if `None`, then it will be substituted with `Tile::Void` **/
    pub fn new_void(piece_raw: Option<Piece>) -> Tile {
        match piece_raw {
            Some(piece) => Tile::Piece(piece),
            _ => Tile::Void,
        }
    }

    /** Returns whether or not there is no piece on that tile. **/
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Tile::Piece(_) => true,
            _ => false
        }
    }

    /** Turns a `Tile` into an `Option<Piece>` (useful if `.into()` can't determine the target type) **/
    #[inline]
    pub fn piece(&self) -> Option<Piece> {
        match self {
            Tile::Piece(piece) => Some(*piece),
            _ => None,
        }
    }

    /** Acts like Option::map **/
    #[inline]
    pub fn map<X, F: Fn(Piece) -> X>(&self, f: F) -> Option<X> {
        self.piece().map(f)
    }
}

impl From<Piece> for Tile {
    fn from(piece: Piece) -> Tile {
        Tile::Piece(piece)
    }
}

impl From<Option<Tile>> for Tile {
    /**
        Since that conversion is mainly used when doing `game.get_board(...).map(|board| board.get(...)).into()`,
        the `None` option is substituted with `Tile::Void`.
    **/
    fn from(tile: Option<Tile>) -> Tile {
        match tile {
            Some(t) => t,
            None => Tile::Void,
        }
    }
}

impl Into<Option<Piece>> for Tile {
    fn into(self) -> Option<Piece> {
        match self {
            Tile::Piece(piece) => Some(piece),
            _ => None,
        }
    }
}
