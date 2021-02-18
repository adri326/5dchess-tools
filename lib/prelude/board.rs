use super::*;
use colored::*;
use std::fmt;

/**
    A board of the game. Contains a set of `width` × `height` pieces and other informations that are useful for processing boards.
**/
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board {
    pub l: Layer,
    pub t: Time,
    width: Physical,
    height: Physical,
    pub pieces: Vec<Tile>,
    pub en_passant: Option<(Physical, Physical)>,
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
    ) -> Self {
        // TODO: use `BitBoardPrimitive::BITS` once it becomes stabilized
        let fits_bitboards = if cfg!(bitboard128) {
            (width as u32) * (height as u32) <= 128
        } else {
            (width as u32) * (height as u32) <= 64
        };

        let bitboards = if fits_bitboards {
            BitBoards::from_pieces(&pieces)
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

/// Slower, but supports boards up to 11x11
#[cfg(bitboard128)]
pub type BitBoardPrimitive = u128;
/// Faster, but only supports boards up to 8x8
#[cfg(not(bitboard128))]
pub type BitBoardPrimitive = u64;

/** The number of bitboards that there are.
    If your pieces can't be expressed using the basic 5D Chess pieces, you'll have to add new bitboards and increase this amount.
    Currently, there are 11 piece movement kinds that are used:
    1. pawn capture
    2. brawn capture (minus pawn capture)
    3. 1-agonal leaper (wazir)
    4. 2-agonal leaper (ferz)
    5. 3-agonal leaper (rhino)
    6. 4-agonal leaper (wolf)
    7. 1-agonal rider (rook)
    8. 2-agonal rider (bishop)
    9. 3-agonal rider (unicorn)
    10. 4-agonal rider (dragon)
    11. ⟨2,1,0,0⟩-leaper (knight)
**/
pub const N_BITBOARDS: usize = 11;

/**
    Contains the bitboards for the different piece kinds of each player.
    They are named after their respective, basic piece movements
**/
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BitBoards {
    // White's pieces
    pub white: [BitBoardPrimitive; N_BITBOARDS],
    pub white_royal: BitBoardPrimitive,
    pub white_movable: BitBoardPrimitive,

    // Black's pieces
    pub black: [BitBoardPrimitive; N_BITBOARDS],
    pub black_royal: BitBoardPrimitive,
    pub black_movable: BitBoardPrimitive,
}

impl BitBoards {
    pub fn set(&mut self, mask: &BitBoardMask, shift: u32) {
        for n in 0..N_BITBOARDS {
            self.white[n] = (self.white[n] & !(1 << shift)) | (mask.white[n] as BitBoardPrimitive) << shift;
            self.black[n] = (self.black[n] & !(1 << shift)) | (mask.black[n] as BitBoardPrimitive) << shift;
        }

        self.white_royal = (self.white_royal & !(1 << shift)) | (mask.white_royal as BitBoardPrimitive) << shift;
        self.black_royal = (self.black_royal & !(1 << shift)) | (mask.black_royal as BitBoardPrimitive) << shift;

        self.white_movable = (self.white_movable & !(1 << shift)) | (mask.white_movable as BitBoardPrimitive) << shift;
        self.black_movable = (self.black_movable & !(1 << shift)) | (mask.black_movable as BitBoardPrimitive) << shift;
    }

    /// Assumes that `pieces` fits!
    pub fn from_pieces(pieces: &Vec<Tile>) -> Self {
        let mut white = [0; N_BITBOARDS];
        let mut white_royal = 0;
        let mut white_movable = 0;

        let mut black = [0; N_BITBOARDS];
        let mut black_royal = 0;
        let mut black_movable = 0;

        for n in 0..(pieces.len() as u32) {
            let mask = pieces[n as usize].bitboard_mask();
            for o in 0..N_BITBOARDS {
                white[o] |= (mask.white[o] as BitBoardPrimitive) << n;
                black[o] |= (mask.black[o] as BitBoardPrimitive) << n;
            }
            white_royal |= (mask.white_royal as BitBoardPrimitive) << n;
            white_movable |= (mask.white_movable as BitBoardPrimitive) << n;
            black_royal |= (mask.black_royal as BitBoardPrimitive) << n;
            black_movable |= (mask.black_movable as BitBoardPrimitive) << n;
        }

        Self {
            white,
            white_royal,
            white_movable,

            black,
            black_royal,
            black_movable,
        }
    }
}

impl Default for BitBoards {
    fn default() -> Self {
        Self {
            // White's pieces
            white: [0; N_BITBOARDS],
            white_royal: 0,
            white_movable: !0,

            // Black's pieces
            black: [0; N_BITBOARDS],
            black_royal: 0,
            black_movable: !0,
        }
    }
}

pub const VOID_BITBOARDS: BitBoards = BitBoards {
    white: [0; N_BITBOARDS],
    white_royal: 0,
    white_movable: 0,
    black: [0; N_BITBOARDS],
    black_royal: 0,
    black_movable: 0,
};

/// Contains the state of a piece, to then be put into a bitboard
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BitBoardMask {
    // White's pieces
    pub white: [bool; N_BITBOARDS],
    pub white_royal: bool,
    pub white_movable: bool,

    // Black's pieces
    pub black: [bool; N_BITBOARDS],
    pub black_royal: bool,
    pub black_movable: bool,
}

impl Default for BitBoardMask {
    fn default() -> Self {
        Self {
            // White's pieces
            white: [false; N_BITBOARDS],
            white_royal: false,
            white_movable: true,

            // Black's pieces
            black: [false; N_BITBOARDS],
            black_royal: false,
            black_movable: true,
        }
    }
}
