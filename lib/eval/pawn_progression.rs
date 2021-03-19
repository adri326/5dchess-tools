use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PawnProgression {
    pub pawn_delta: Eval,
    pub brawn_delta: Eval,

    pub inactive_multiplier: Eval,
}

impl PawnProgression {
    pub fn pawn_delta(mut self, value: Eval) -> Self {
        self.pawn_delta = value;
        self
    }

    pub fn brawn_delta(mut self, value: Eval) -> Self {
        self.brawn_delta = value;
        self
    }

    pub fn inactive_multiplier(mut self, value: Eval) -> Self {
        self.inactive_multiplier = value;
        self
    }
}

impl Default for PawnProgression {
    fn default() -> Self {
        Self {
            pawn_delta: 0.1,
            brawn_delta: 0.1,

            inactive_multiplier: 0.25,
        }
    }
}

impl EvalBoardFn for PawnProgression {
    fn eval_board(&self, _game: &Game, node: &TreeNode, board: &Board) -> Option<Eval> {
        let partial_game = &node.partial_game;
        let mut sum: Eval = 0.0;

        let multiplier = if partial_game.info.is_active(board.l()) { 1.0 } else { self.inactive_multiplier };

        for (index, piece) in board.pieces.iter().enumerate() {
            if let Tile::Piece(piece) = piece {
                let y = (index % board.width() as usize) as Physical;
                let base_value = if piece.kind == PieceKind::Pawn {
                    self.pawn_delta
                } else if piece.kind == PieceKind::Brawn {
                    self.brawn_delta
                } else {
                    continue
                };
                if piece.white {
                    sum += base_value * y as Eval * multiplier;
                } else {
                    sum -= base_value * (board.height() - y - 1) as Eval * multiplier;
                }
            }
        }

        if !partial_game.info.active_player {
            sum = -sum;
        }

        Some(sum)
    }
}
