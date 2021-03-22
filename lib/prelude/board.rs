use super::*;
use colored::*;
use std::fmt;

/**
    A board of the game. Contains a set of `width` × `height` pieces and other informations that are useful for processing boards.

    If you mutate a Board, you will also need to make sure that all of the informations are updated accordingly.
    You should rely on functions like `Move::generate_source_board` to correctly update these informations.
**/
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board {
    pub l: Layer,
    pub t: Time,
    width: Physical,
    height: Physical,
    pub pieces: Vec<Tile>,
    pub en_passant: Option<(Physical, Physical)>,
    pub castle: Option<(Physical, Physical, Physical, Physical)>,
    pub bitboards: BitBoards,
    fits_bitboards: bool,
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
    /**
        Creates a new board instance.
        The bitboards will be filled according to `pieces`.
        You should consider cloning the board if you wish to make changes to it.
    **/
    pub fn new(
        width: Physical,
        height: Physical,
        l: Layer,
        t: Time,
        pieces: Vec<Tile>,
        en_passant: Option<(Physical, Physical)>,
        castle: Option<(Physical, Physical, Physical, Physical)>,
    ) -> Self {
        // TODO: use `BitBoardPrimitive::BITS` once it becomes stabilized
        let fits_bitboards = if cfg!(bitboard128) {
            (width as u32) * (height as u32) <= 128
        } else {
            (width as u32) * (height as u32) <= 64
        };

        let bitboards = if fits_bitboards {
            let mut res = BitBoards::from_pieces(&pieces);
            if let Some((x1, y1, x2, y2)) = castle {
                println!("{} {} {} {}", x1, y1, x2, y2);
                res.set_castle(Some((
                    x1 as u32 + y1 as u32 * width as u32,
                    x2 as u32 + y2 as u32 * width as u32
                )));
            }
            res
        } else {
            BitBoards::default()
        };

        Board {
            width,
            height,
            l,
            t,
            pieces,
            en_passant,
            bitboards,
            fits_bitboards,
            castle,
        }
    }

    /** Returns the timeline coordinate of the board. **/
    #[inline]
    pub fn l(&self) -> Layer {
        self.l
    }

    /** Returns the time coordinate of the board. **/
    #[inline]
    pub fn t(&self) -> Time {
        self.t
    }

    /** Returns the timeline and time coordinates of the board. **/
    #[inline]
    pub fn non_physical(&self) -> (Layer, Time) {
        (self.l, self.t)
    }

    /** Returns the width of the board. **/
    #[inline]
    pub fn width(&self) -> Physical {
        self.width
    }

    /** Returns the height of the board. **/
    #[inline]
    pub fn height(&self) -> Physical {
        self.height
    }

    /** Returns the en_passant value for the board. **/
    #[inline]
    pub fn en_passant(&self) -> Option<(Physical, Physical)> {
        self.en_passant
    }

    /** Sets the en_passant value for the board. **/
    #[inline]
    pub fn set_en_passant(&mut self, en_passant: Option<(Physical, Physical)>) {
        self.en_passant = en_passant
    }

    /** Returns the castling information for the board. **/
    #[inline]
    pub fn castle(&self) -> Option<(Physical, Physical, Physical, Physical)> {
        self.castle
    }

    /** Sets the castling information for the board. **/
    #[inline]
    pub fn set_castle(&mut self, castle: Option<(Physical, Physical, Physical, Physical)>) {
        self.castle = castle;
        if self.fits_bitboards {
            if let Some((x1, y1, x2, y2)) = castle {
                self.bitboards.set_castle(Some((
                    x1 as u32 + y1 as u32 * self.width as u32,
                    x2 as u32 + y2 as u32 * self.width as u32
                )));
            } else {
                self.bitboards.set_castle(None);
            }
        }
    }

    /** Get the piece at (x, y); return Tile::Void if the coordinates aren't within the board. **/
    #[inline]
    pub fn get(&self, (x, y): (Physical, Physical)) -> Tile {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            Tile::Void
        } else {
            self.pieces[(x + self.width * y) as usize].into()
        }
    }

    /** Get the piece at (x, y); panics if it doesn't fit. **/
    #[inline]
    pub fn get_unchecked(&self, (x, y): (Physical, Physical)) -> Tile {
        self.pieces[(x + self.width * y) as usize]
    }

    /** Returns true if the board belongs to white, false otherwise. **/
    #[inline]
    pub fn white(&self) -> bool {
        self.t % 2 == 0
    }

    /** Returns whether or not the board is active and must be played on. **/
    #[inline]
    pub fn active(&self, info: &Info) -> bool {
        info.is_active(self.l) && info.present >= self.t
    }

    /** Sets the piece at (x, y); return Some(()) if the coordinates are within the board, None otherwise **/
    #[inline]
    pub fn set(&mut self, (x, y): (Physical, Physical), tile: Tile) -> Option<()> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            let index = x as usize + self.width as usize * y as usize;
            self.pieces[index] = tile;

            if self.fits_bitboards {
                self.bitboards.set(tile.bitboard_mask(), index as u32);
            }

            Some(())
        } else {
            None
        }
    }

    /** Sets the piece at (x, y), without verifying that x and y fit. **/
    #[inline]
    pub fn set_unchecked(&mut self, (x, y): (Physical, Physical), tile: Tile) {
        let index = x as usize + self.width as usize * y as usize;
        self.pieces[index] = tile;

        if self.fits_bitboards {
            self.bitboards.set(tile.bitboard_mask(), index as u32);
        }
    }
}
