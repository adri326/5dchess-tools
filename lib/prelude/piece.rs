use colored::*;
use std::fmt;

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
