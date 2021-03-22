use colored::*;
use std::fmt;
use crate::prelude::{BitBoardMask, PIECE_MASKS};

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

/// The number of piece kinds (`||PieceKind||`)
pub const N_PIECES: usize = 12;

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
    /**
        Creates a new `Piece` instance.
    **/
    pub fn new(kind: PieceKind, white: bool, moved: bool) -> Self {
        Self { kind, white, moved }
    }

    /**
        Returns `true` if the piece is a royal piece (king or royal queen), `false` otherwise.
    **/
    #[inline]
    pub fn is_royal(&self) -> bool {
        match self.kind {
            PieceKind::King | PieceKind::RoyalQueen => true,
            _ => false,
        }
    }

    /**
        Returns `true` if the piece can castle (king), `false` otherwise.
    **/
    #[inline]
    pub fn can_castle(&self) -> bool {
        self.kind == PieceKind::King
    }

    /**
        Returns `true` if the piece can be castled to (rook), `false` otherwise.
    **/
    #[inline]
    pub fn can_castle_to(&self) -> bool {
        self.kind == PieceKind::Rook
    }

    /**
        Returns `true` if the piece can promote (pawns and brawns), `false` otherwise.

        *Note:* the promotion rule is hard-coded into `Move::new` (the piece must reach the last row).
    **/
    #[inline]
    pub fn can_promote(&self) -> bool {
        match self.kind {
            PieceKind::Pawn | PieceKind::Brawn => true,
            _ => false,
        }
    }

    /**
        Returns `true` if the piece can capture a kickstarting piece en-passant (pawns and brawns), `false` otherwise.
        The corresponding movement can be yielded if the board's `en_passant` field is set to `Some((x, y))`.

        *Note:* It is currently assumed that no royal piece can kickstart.
    **/
    #[inline]
    pub fn can_enpassant(&self) -> bool {
        match self.kind {
            PieceKind::Pawn | PieceKind::Brawn => true,
            _ => false,
        }
    }

    /**
        Returns `true` if the piece can "kickstart" if it hadn't been moved yet.
    **/
    #[inline]
    pub fn can_kickstart(&self) -> bool {
        match self.kind {
            PieceKind::Pawn | PieceKind::Brawn => true,
            _ => false,
        }
    }

    /**
        Returns the bitboard mask corresponding to that piece.
    **/
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
                PieceKind::Brawn => "W",
                PieceKind::Unicorn => "U",
                PieceKind::Dragon => "D",
                PieceKind::CommonKing => "CK",
                PieceKind::RoyalQueen => "RQ",
            }
        )
    }
}
