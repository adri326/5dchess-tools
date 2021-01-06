use super::*;

/** Represents a move's kind (regular move, castling move, etc.) **/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveKind {
    Normal,
    Castle,
    EnPassant,
    Promotion
}

/** Represents a piece's movement. **/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: (Piece, Coords),
    pub to: (Option<Piece>, Coords),
    pub kind: MoveKind,
}

impl Move {
    /** Creates a new move; the move's kind is deduced from the coordinates and the game state. **/
    pub fn new(game: &Game, from: Coords, to: Coords) -> Option<Self> {
        let mut kind = MoveKind::Normal;
        let from = (game.get(from)?, from);
        let to = (game.get(to), to);

        if from.0.can_enpassant() && to.0.is_none() && (from.1).2 != (to.1).2 {
            kind = MoveKind::EnPassant;
        } else if from.0.can_promote() && ((to.1).2 == 0 && (from.1).2 != 0 || (to.1).2 == game.height - 1 && (from.1).2 != game.height - 1) {
            kind = MoveKind::Promotion;
        } else if from.0.can_castle() && ((from.1).2 == (to.1).2 + 2 || (from.1).2 + 2 == (to.1).2) {
            kind = MoveKind::Castle;
        }

        Some(Self {
            from,
            to,
            kind
        })
    }

    #[inline]
    pub fn captures(&self) -> bool {
        self.to.0.is_some()
    }
}
