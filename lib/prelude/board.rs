use std::fmt;
use colored::*;
use super::*;

pub struct Board {
    pub l: Layer,
    pub t: Time,
    pub width: Physical,
    pub height: Physical,
    pub pieces: Vec<Option<Piece>>,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n")?;
        for y in 0..self.height {
            for x in 0..self.width {
                match self.get((x, y)) {
                    Some(x) => write!(f, "{:?}", x)?,
                    None => write!(f, "{}", ".".white())?,
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Board {
    pub fn new(width: Physical, height: Physical, l: Layer, t: Time, pieces: Vec<Option<Piece>>) -> Self {
        Board {
            width,
            height,
            l,
            t,
            pieces,
        }
    }

    pub fn get(&self, (x, y): (Physical, Physical)) -> Option<Piece> {
        self.pieces.get((x + self.width * y) as usize).map(|x| *x).flatten()
    }

    pub fn get_unchecked(&self, (x, y): (Physical, Physical)) -> Option<Piece> {
        self.pieces[(x + self.width * y) as usize]
    }
}
