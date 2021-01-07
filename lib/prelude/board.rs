use std::fmt;
use colored::*;
use super::*;

pub struct Board {
    pub l: Layer,
    pub t: Time,
    pub width: Physical,
    pub height: Physical,
    pub pieces: Vec<Tile>,
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

impl Board {
    pub fn new(width: Physical, height: Physical, l: Layer, t: Time, pieces: Vec<Tile>) -> Self {
        Board {
            width,
            height,
            l,
            t,
            pieces,
        }
    }

    pub fn get(&self, (x, y): (Physical, Physical)) -> Tile {
        self.pieces.get((x + self.width * y) as usize).map(|x| *x).into()
    }

    pub fn get_unchecked(&self, (x, y): (Physical, Physical)) -> Tile {
        self.pieces[(x + self.width * y) as usize]
    }
}
