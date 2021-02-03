use super::*;
use colored::*;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board {
    pub l: Layer,
    pub t: Time,
    width: Physical,
    height: Physical,
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
            if y == 0 {
                write!(f, " {}", format!("@({}:{})", self.l, self.t).white())?;
                match self.en_passant {
                    Some((x, y)) => write!(f, "{}", format!("/ep.({}:{})", x, y).white())?,
                    None => write!(f, "{}", format!("/no ep").white())?,
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
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

    #[inline]
    pub fn get(&self, (x, y): (Physical, Physical)) -> Tile {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            Tile::Void
        } else {
            self.pieces[(x + self.width * y) as usize].into()
            // .get((x + self.width * y) as usize)
            // .map(|x| *x)
            // .into()
        }
    }

    #[inline]
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
