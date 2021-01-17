use super::*;
use colored::*;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    l: Layer,
    t: Time,
    width: Physical,
    height: Physical,
    pub pieces: Vec<Tile>,
    en_passant: Option<(Physical, Physical)>,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n")?;
        for y in 0..self.height {
            for x in 0..self.width {
                match self.get((x, y)) {
                    Tile::Piece(x) => write!(f, "{:?}", x)?,
                    _ => write!(f, "{}", ".".white())?,
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl std::convert::AsRef<Board> for Board {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Board {
    pub fn new(
        width: Physical,
        height: Physical,
        l: Layer,
        t: Time,
        pieces: Vec<Tile>,
        en_passant: Option<(Physical, Physical)>,
    ) -> Self {
        Board {
            width,
            height,
            l,
            t,
            pieces,
            en_passant,
        }
    }

    #[inline]
    pub fn l(&self) -> Layer {
        self.l
    }

    #[inline]
    pub fn t(&self) -> Time {
        self.t
    }

    #[inline]
    pub fn width(&self) -> Physical {
        self.width
    }

    #[inline]
    pub fn height(&self) -> Physical {
        self.height
    }

    #[inline]
    pub fn en_passant(&self) -> Option<(Physical, Physical)> {
        self.en_passant
    }

    #[inline]
    pub fn set_en_passant(&mut self, en_passant: Option<(Physical, Physical)>) {
        self.en_passant = en_passant
    }

    pub fn get(&self, (x, y): (Physical, Physical)) -> Tile {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            Tile::Void
        } else {
            self.pieces
                .get((x + self.width * y) as usize)
                .map(|x| *x)
                .into()
        }
    }

    pub fn get_unchecked(&self, (x, y): (Physical, Physical)) -> Tile {
        self.pieces[(x + self.width * y) as usize]
    }

    #[inline]
    pub fn white(&self) -> bool {
        self.t % 2 == 0
    }

    #[inline]
    pub fn active(&self, info: &Info) -> bool {
        info.is_active(self.l) && info.present >= self.t
    }
}

pub enum BoardOr<'a, B: Clone + AsRef<Board> + 'a> {
    Board(&'a Board),
    B(&'a B),
}

impl<'a, B: Clone + AsRef<Board> + 'a> BoardOr<'a, B> {
    #[inline]
    pub fn l(&self) -> Layer {
        self.as_ref().l()
    }

    #[inline]
    pub fn t(&self) -> Time {
        self.as_ref().t()
    }

    #[inline]
    pub fn width(&self) -> Physical {
        self.as_ref().width()
    }

    #[inline]
    pub fn height(&self) -> Physical {
        self.as_ref().height()
    }

    #[inline]
    pub fn get(&self, (x, y): (Physical, Physical)) -> Tile {
        self.as_ref().get((x, y))
    }

    #[inline]
    pub fn get_unchecked(&self, (x, y): (Physical, Physical)) -> Tile {
        self.as_ref().get_unchecked((x, y))
    }

    #[inline]
    pub fn white(&self) -> bool {
        self.as_ref().white()
    }

    #[inline]
    pub fn active(&self, info: &Info) -> bool {
        self.as_ref().active(info)
    }

    #[inline]
    pub fn en_passant(&self) -> Option<(Physical, Physical)> {
        self.as_ref().en_passant
    }
}

impl<'a, B: Clone + AsRef<Board> + 'a> From<BoardOr<'a, B>> for Board {
    fn from(borb: BoardOr<B>) -> Board {
        match borb {
            BoardOr::Board(board) => board.clone(),
            BoardOr::B(board) => board.as_ref().clone(),
        }
    }
}

impl<'a, B: Clone + AsRef<Board> + 'a> std::convert::AsRef<Board> for BoardOr<'a, B> {
    #[inline]
    fn as_ref(&self) -> &Board {
        match &self {
            BoardOr::Board(board) => board,
            BoardOr::B(board) => board.as_ref(),
        }
    }
}

impl<'a, B: Clone + AsRef<Board> + 'a> std::convert::From<&'a Board> for BoardOr<'a, B> {
    fn from(board: &'a Board) -> Self {
        BoardOr::Board(board)
    }
}
