use colored::*;
use std::fmt;
use crate::board::{BitBoardMask, N_BITBOARDS};

/**
    Represents the kind of a piece (pawn, knight, etc.)
**/
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    Princess,
    King,
    Brawn,
    Unicorn,
    Dragon,
    CommonKing,
    RoyalQueen,
}

/**
    Represents a piece within the game, stores its kind, its color and whether or not it had already moved.
**/
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Piece {
    pub kind: PieceKind,
    pub white: bool,
    pub moved: bool,
}

impl Piece {
    pub fn new(kind: PieceKind, white: bool, moved: bool) -> Self {
        Self { kind, white, moved }
    }

    #[inline]
    pub fn is_royal(&self) -> bool {
        match self.kind {
            PieceKind::King | PieceKind::RoyalQueen => true,
            _ => false,
        }
    }

    #[inline]
    pub fn can_castle(&self) -> bool {
        self.kind == PieceKind::King
    }

    #[inline]
    pub fn can_castle_to(&self) -> bool {
        self.kind == PieceKind::Rook
    }

    #[inline]
    pub fn can_promote(&self) -> bool {
        match self.kind {
            PieceKind::Pawn | PieceKind::Brawn => true,
            _ => false,
        }
    }

    #[inline]
    pub fn can_enpassant(&self) -> bool {
        match self.kind {
            PieceKind::Pawn | PieceKind::Brawn => true,
            _ => false,
        }
    }

    #[inline]
    pub fn can_kickstart(&self) -> bool {
        match self.kind {
            PieceKind::Pawn | PieceKind::Brawn => true,
            _ => false,
        }
    }

    #[inline]
    pub fn bitboard_mask(&self) -> &'static BitBoardMask {
        match self.kind {
            // Indices start at 2 as 0 is the Blank matrix and 1 the Void matrix
            PieceKind::Pawn => &PIECE_MASKS[(self.white as usize * N_PIECES) + 2],
            PieceKind::Knight => &PIECE_MASKS[(self.white as usize * N_PIECES) + 3],
            PieceKind::Bishop => &PIECE_MASKS[(self.white as usize * N_PIECES) + 4],
            PieceKind::Rook => &PIECE_MASKS[(self.white as usize * N_PIECES) + 5],
            PieceKind::Queen => &PIECE_MASKS[(self.white as usize * N_PIECES) + 6],
            PieceKind::Princess => &PIECE_MASKS[(self.white as usize * N_PIECES) + 7],
            PieceKind::King => &PIECE_MASKS[(self.white as usize * N_PIECES) + 8],
            PieceKind::Brawn => &PIECE_MASKS[(self.white as usize * N_PIECES) + 9],
            PieceKind::Unicorn => &PIECE_MASKS[(self.white as usize * N_PIECES) + 10],
            PieceKind::Dragon => &PIECE_MASKS[(self.white as usize * N_PIECES) + 11],
            PieceKind::CommonKing => &PIECE_MASKS[(self.white as usize * N_PIECES) + 12],
            PieceKind::RoyalQueen => &PIECE_MASKS[(self.white as usize * N_PIECES) + 13]
        }
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self.kind);
        if self.moved {
            if self.white {
                write!(f, "{}", s.green().bold())
            } else {
                write!(f, "{}", s.magenta().bold())
            }
        } else {
            if self.white {
                write!(f, "{}", s.green())
            } else {
                write!(f, "{}", s.magenta())
            }
        }
    }
}

impl fmt::Debug for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PieceKind::Pawn => "P",
                PieceKind::Knight => "N",
                PieceKind::Bishop => "B",
                PieceKind::Rook => "R",
                PieceKind::Queen => "Q",
                PieceKind::Princess => "S",
                PieceKind::King => "K",
                PieceKind::Brawn => "β",
                PieceKind::Unicorn => "U",
                PieceKind::Dragon => "D",
                PieceKind::CommonKing => "κ",
                PieceKind::RoyalQueen => "ρ",
            }
        )
    }
}

pub const N_PIECES: usize = 12;

lazy_static! {
    pub static ref PIECE_MASKS: [BitBoardMask; 2 * N_PIECES + 2] = {
        let mut res: [BitBoardMask; 2 * N_PIECES + 2] = [BitBoardMask::default(); 2 * N_PIECES + 2];
        res[1].white_movable = false;
        res[1].black_movable = false;

        // Number wall goes brr
        let kernel: [([u8; N_BITBOARDS], u8); N_PIECES] = [
            ([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 0),
            ([0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0], 0),
            ([0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0], 0),
            ([0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0], 1),
            ([1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0], 0),
            ([0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0], 1),
        ];

        for (i, k) in kernel.iter().enumerate() {
            let mut transformed_kernel: [bool; N_BITBOARDS] = [false; N_BITBOARDS];

            for n in 0..N_BITBOARDS {
                transformed_kernel[n] = k.0[n] > 0;
            }

            // White
            res[i + N_PIECES + 2] = BitBoardMask {
                white: transformed_kernel,
                white_royal: k.1 > 0,
                white_movable: false,
                black: [false; N_BITBOARDS],
                black_royal: false,
                black_movable: true,
            };

            // Black
            res[i + 2] = BitBoardMask {
                white: [false; N_BITBOARDS],
                white_royal: false,
                white_movable: false,
                black: transformed_kernel,
                black_royal: k.1 > 0,
                black_movable: true,
            };
        }

        res
    };
}
