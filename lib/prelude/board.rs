use super::*;
use colored::*;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    pub l: Layer,
    pub t: Time,
    pub width: Physical,
    pub height: Physical,
    pub pieces: Vec<Tile>,
    pub en_passant: Option<(Physical, Physical)>,
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
