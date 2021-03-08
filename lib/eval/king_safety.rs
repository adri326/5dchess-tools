use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KingSafety {
    pub allowed_distance: Physical,
    pub orthogonal_empty: Eval,
    pub orthogonal_opponent: Eval,
    pub diagonal_empty: Eval,
    pub diagonal_opponent: Eval,

    pub additional_king: Eval,

    pub inactive_multiplier: Eval,
    // TODO: triagonals?
}

impl KingSafety {
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

    pub fn inactive_multiplier(mut self, value: Eval) -> Self {
        self.inactive_multiplier = value;
        self
    }

    pub fn additional_king(mut self, value: Eval) -> Self {
        self.additional_king = value;
        self
    }
}

impl Default for KingSafety {
    fn default() -> Self {
        Self {
            allowed_distance: 1,
            orthogonal_empty: -4.0,
            orthogonal_opponent: -6.0,
            diagonal_empty: -2.0,
            diagonal_opponent: -4.0,

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

impl EvalFn for KingSafety {
    fn eval<'a>(&self, game: &'a Game, node: &'a TreeNode) -> Option<Eval> {
        let partial_game = &node.partial_game;
        let mut sum: Eval = 0.0;

        let mut found_king_white: bool = false;
        let mut found_king_black: bool = false;

        for board in partial_game.own_boards(game).chain(partial_game.opponent_boards(game)) {
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
        }

        if !partial_game.info.active_player {
            sum = -sum;
        }

        Some(sum)
    }
}
