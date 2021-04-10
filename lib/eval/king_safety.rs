use super::*;

/**
    Gives a score based on the safety of the king. Gives penalties if a king's orthogonals or diagonals are open or occupied by an opponent's piece.
    Can be used to approximate check penalty.
**/
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KingSafety2D {
    /// The maximum distance that a defending pieces on the king's orthogonals or diagonals can be, beyond which the ortogonal or diagonal is considered empty (, default `1`)
    pub allowed_distance: Physical,
    /// Evaluation value if a 1D orthogonal is empty (lower gives a stronger penalty, default `-4.0`)
    pub orthogonal_empty: Eval,
    /// Evaluation value if a 1D orthogonal is occupied by an opponent's piece (lower gives a stronger penalty, default `-6.0`)
    pub orthogonal_opponent: Eval,
    /// Evaluation value if a 2D diagonal is empty (lower gives a stronger penalty, default `-2.0`)
    pub diagonal_empty: Eval,
    /// Evaluation value if a 2D diagonal is occupied by an opponent's pieces (lower gives a stronger penalty, default `-4.0`)
    pub diagonal_opponent: Eval,

    /// Evaluation value if a 2D knight-jump is empty (ignores allowed_distance, lower gives a stronger penalty, default `0.0`)
    pub knight_empty: Eval,
    /// Evaluation value if a 2D knight-jump is occupied by an opponent's pieces (ignores allowed_distance, lower gives a stronger penalty, default `-3.0`)
    pub knight_opponent: Eval,

    /// Evaluation value if there are more than one king on one board (lower gives a stronger penalty, default `-4.0`)
    pub additional_king: Eval,

    /// Multiplier for inactive timelines (default `0.25`)
    pub inactive_multiplier: Eval,
}
// TODO: triagonals? -> in a KingSafety4D struct

impl KingSafety2D {
    pub fn allowed_distance(mut self, value: Physical) -> Self {
        self.allowed_distance = value;
        self
    }

    pub fn orthogonal_empty(mut self, value: Eval) -> Self {
        self.orthogonal_empty = value;
        self
    }

    pub fn orthogonal_opponent(mut self, value: Eval) -> Self {
        self.orthogonal_opponent = value;
        self
    }

    pub fn diagonal_empty(mut self, value: Eval) -> Self {
        self.diagonal_empty = value;
        self
    }

    pub fn diagonal_opponent(mut self, value: Eval) -> Self {
        self.diagonal_opponent = value;
        self
    }

    pub fn knight_empty(mut self, value: Eval) -> Self {
        self.knight_empty = value;
        self
    }

    pub fn knight_opponent(mut self, value: Eval) -> Self {
        self.knight_opponent = value;
        self
    }

    pub fn inactive_multiplier(mut self, value: Eval) -> Self {
        self.inactive_multiplier = value;
        self
    }

    pub fn additional_king(mut self, value: Eval) -> Self {
        self.additional_king = value;
        self
    }
}

impl Default for KingSafety2D {
    fn default() -> Self {
        Self {
            allowed_distance: 1,
            orthogonal_empty: -4.0,
            orthogonal_opponent: -6.0,
            diagonal_empty: -2.0,
            diagonal_opponent: -4.0,
            knight_empty: 0.0,
            knight_opponent: -3.0,

            additional_king: -4.0,

            inactive_multiplier: 0.25,
        }
    }
}

#[macro_use]
mod macros {
    macro_rules! king_safety {
        (
            $self:expr,
            $board:expr,
            $sum:expr,
            $king:expr,
            $index:expr,
            $multiplier:expr,
            $dx:expr,
            $dy:expr,
            $allowed_distance:expr,
            $orthogonal:expr
        ) => {
            let x = ($index % $board.width() as usize) as Physical;
            let y = ($index / $board.height() as usize) as Physical;
            let mut found: bool = false;
            let mut opponent: bool = false;
            for n in 1..=($allowed_distance + 1) {
                match $board.get((x + $dx * n, y + $dy * n)) {
                    Tile::Piece(piece) => {
                        found = true;
                        opponent = piece.white != $king.white;
                        break
                    }
                    Tile::Blank => {},
                    Tile::Void => {
                        found = true;
                        break
                    }
                }
            }
            let res = if found && opponent {
                if $orthogonal {
                    $self.orthogonal_opponent
                } else {
                    $self.diagonal_opponent
                }
            } else if !found {
                if $orthogonal {
                    $self.orthogonal_empty
                } else {
                    $self.diagonal_empty
                }
            } else {
                0.0
            };
            $sum += res * (if $king.white { 1.0 } else { -1.0 }) * $multiplier;
        }
    }
}

impl EvalBoardFn for KingSafety2D {
    fn eval_board(&self, _game: &Game, node: &TreeNode, board: &Board) -> Option<Eval> {
        let partial_game = &node.partial_game;
        let mut sum: Eval = 0.0;

        let mut found_king_white: bool = false;
        let mut found_king_black: bool = false;
        let multiplier = if partial_game.info.is_active(board.l()) { 1.0 } else { self.inactive_multiplier };
        for (index, piece) in board.pieces.iter().enumerate() {
            if let Tile::Piece(piece) = piece {
                if piece.kind == PieceKind::King {
                    if self.orthogonal_empty != 0.0 || self.orthogonal_opponent != 0.0 {
                        king_safety!(self, board, sum, piece, index, multiplier, 0, 1, self.allowed_distance, true);
                        king_safety!(self, board, sum, piece, index, multiplier, 0, -1, self.allowed_distance, true);
                        king_safety!(self, board, sum, piece, index, multiplier, 1, 0, self.allowed_distance, true);
                        king_safety!(self, board, sum, piece, index, multiplier, -1, 0, self.allowed_distance, true);
                    }
                    if self.diagonal_empty != 0.0 || self.diagonal_opponent != 0.0 {
                        king_safety!(self, board, sum, piece, index, multiplier, 1, 1, self.allowed_distance, true);
                        king_safety!(self, board, sum, piece, index, multiplier, 1, -1, self.allowed_distance, true);
                        king_safety!(self, board, sum, piece, index, multiplier, -1, 1, self.allowed_distance, true);
                        king_safety!(self, board, sum, piece, index, multiplier, -1, -1, self.allowed_distance, true);
                    }
                    if self.knight_empty != 0.0 || self.knight_opponent != 0.0 {
                        king_safety!(self, board, sum, piece, index, multiplier, 2, 1, 0, true);
                        king_safety!(self, board, sum, piece, index, multiplier, -2, 1, 0, true);
                        king_safety!(self, board, sum, piece, index, multiplier, 2, -1, 0, true);
                        king_safety!(self, board, sum, piece, index, multiplier, -2, 1, 0, true);

                        king_safety!(self, board, sum, piece, index, multiplier, 1, 2, 0, true);
                        king_safety!(self, board, sum, piece, index, multiplier, -1, 2, 0, true);
                        king_safety!(self, board, sum, piece, index, multiplier, 1, -2, 0, true);
                        king_safety!(self, board, sum, piece, index, multiplier, -1, 2, 0, true);
                    }

                    if piece.white {
                        if found_king_white {
                            sum += self.additional_king;
                        } else {
                            found_king_white = true;
                        }
                    } else {
                        if found_king_black {
                            sum -= self.additional_king;
                        } else {
                            found_king_black = true;
                        }
                    }
                }
            }
        }

        if !partial_game.info.active_player {
            sum = -sum;
        }

        Some(sum)
    }
}
