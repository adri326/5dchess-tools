use super::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PieceValues {
    pub pawn: Eval,
    pub brawn: Eval,
    pub knight: Eval,
    pub rook: Eval,
    pub bishop: Eval,
    pub unicorn: Eval,
    pub dragon: Eval,
    pub queen: Eval,
    pub royal_queen: Eval,
    pub princess: Eval,
    pub king: Eval,
    pub common_king: Eval,

    pub inactive_multiplier: Eval,
}

impl PieceValues {
    pub fn pawn(mut self, value: Eval) -> Self {
        self.pawn = value;
        self
    }

    pub fn brawn(mut self, value: Eval) -> Self {
        self.brawn = value;
        self
    }

    pub fn knight(mut self, value: Eval) -> Self {
        self.knight = value;
        self
    }

    pub fn rook(mut self, value: Eval) -> Self {
        self.rook = value;
        self
    }

    pub fn bishop(mut self, value: Eval) -> Self {
        self.bishop = value;
        self
    }

    pub fn unicorn(mut self, value: Eval) -> Self {
        self.unicorn = value;
        self
    }

    pub fn dragon(mut self, value: Eval) -> Self {
        self.dragon = value;
        self
    }

    pub fn queen(mut self, value: Eval) -> Self {
        self.queen = value;
        self
    }

    pub fn royal_queen(mut self, value: Eval) -> Self {
        self.royal_queen = value;
        self
    }

    pub fn princess(mut self, value: Eval) -> Self {
        self.princess = value;
        self
    }

    pub fn king(mut self, value: Eval) -> Self {
        self.king = value;
        self
    }

    pub fn common_king(mut self, value: Eval) -> Self {
        self.common_king = value;
        self
    }

    pub fn inactive_multiplier(mut self, value: Eval) -> Self {
        self.inactive_multiplier = value;
        self
    }
}

impl Default for PieceValues {
    fn default() -> Self {
        Self {
            pawn: 1.0,
            brawn: 1.5,
            knight: 3.5,
            rook: 3.0,
            bishop: 5.0,
            unicorn: 2.5,
            dragon: 1.5,
            queen: 12.0,
            princess: 9.0,
            royal_queen: 9.0,
            king: -4.0,
            common_king: 3.0,

            inactive_multiplier: 0.25,
        }
    }
}

impl EvalFn for PieceValues {
    fn eval<'a>(&self, game: &'a Game, node: &'a TreeNode) -> Option<Eval> {
        let partial_game = &node.partial_game;
        let mut sum: Eval = 0.0;
        for board in partial_game.own_boards(game).chain(partial_game.opponent_boards(game)) {
            let multiplier = if partial_game.info.is_active(board.l()) { 1.0 } else { self.inactive_multiplier };
            for piece in &board.pieces {
                if let Tile::Piece(piece) = piece {
                    let value = match piece.kind {
                        PieceKind::Pawn => self.pawn,
                        PieceKind::Knight => self.knight,
                        PieceKind::Bishop => self.bishop,
                        PieceKind::Rook => self.rook,
                        PieceKind::Queen => self.queen,
                        PieceKind::Princess => self.princess,
                        PieceKind::King => self.king,
                        PieceKind::Brawn => self.brawn,
                        PieceKind::Unicorn => self.unicorn,
                        PieceKind::Dragon => self.dragon,
                        PieceKind::CommonKing => self.common_king,
                        PieceKind::RoyalQueen => self.royal_queen,
                    };
                    sum += if piece.white { 1.0 } else { -1.0 } * value * multiplier;
                }
            }
        }

        if !partial_game.info.active_player {
            sum = -sum;
        }

        Some(sum)
    }
}
