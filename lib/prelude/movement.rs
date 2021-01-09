use super::*;
use std::collections::HashMap;
use std::convert::TryFrom;

/** Represents a move's kind (regular move, castling move, etc.) **/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveKind {
    Normal,
    Castle,
    EnPassant,
    Promotion,
}

/** Represents a piece's movement. **/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: (Piece, Coords),
    pub to: (Option<Piece>, Coords),
    pub kind: MoveKind,
}

impl Move {
    /**
        Creates a new move; the move's kind is deduced from the coordinates and the game state.
        Checks that:
        - there is a piece that is moved
        - the piece moved is of the active player's color
    **/
    pub fn new<B: Clone + AsRef<Board>>(
        game: &Game,
        partial_game: &PartialGame<B>,
        from: Coords,
        to: Coords,
    ) -> Option<Self> {
        let mut kind = MoveKind::Normal;
        let from: (Piece, Coords) = (partial_game.get_with_game(game, from).piece()?, from);
        let to: (Option<Piece>, Coords) = (partial_game.get_with_game(game, to).piece().into(), to);

        if from.0.white != partial_game.info.active_player {
            return None;
        }

        if from.0.can_enpassant() && to.0.is_none() && (from.1).2 != (to.1).2 {
            kind = MoveKind::EnPassant;
        } else if from.0.can_promote()
            && ((to.1).2 == 0 && (from.1).2 != 0
                || (to.1).2 == game.height - 1 && (from.1).2 != game.height - 1)
        {
            kind = MoveKind::Promotion;
        } else if from.0.can_castle() && ((from.1).2 == (to.1).2 + 2 || (from.1).2 + 2 == (to.1).2)
        {
            kind = MoveKind::Castle;
        }

        Some(Self { from, to, kind })
    }

    #[inline]
    pub fn captures(&self) -> bool {
        self.to.0.is_some()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MovesetValidityErr {
    NoMoves,
    TooManyMoves,
    AlreadyPlayed(Move),
    MoveToVoid(Move),
    MoveFromVoid(Move),
    MoveNotFromLast(Move),
    OpponentBoard(Move),
}

/** A set of moves, that is guaranteed to be valid (ie. that could be made).
    Piece-specific checks aren't implemented yet.
    Such a moveset doesn't have to be legal (ie. letting you pass the turn with no checks).
    A move is considered valid if:
    - if all of the moves land on existing boards
    - if all of the moves start from the last board of an existing timeline
    - if no move start from an already-played board
    - if no move start/end on one of the opponent's board
    - there aren't too many moves or no moves
**/
#[derive(Debug, Clone)]
pub struct Moveset {
    moves: Vec<Move>,
}

impl Moveset {
    /** Creates a new moveset from a set of moves and an info, verifying the requirements of the type. **/
    pub fn new(moves: Vec<Move>, info: &Info) -> Result<Moveset, MovesetValidityErr> {
        Self::try_from((moves, info))
    }

    pub fn moves(&self) -> &Vec<Move> {
        &self.moves
    }
}

impl TryFrom<(Vec<Move>, &Info)> for Moveset {
    type Error = MovesetValidityErr;

    /** Creates a new moveset from a set of moves and an info, verifying the requirements of the type. **/
    fn try_from((moves, info): (Vec<Move>, &Info)) -> Result<Moveset, MovesetValidityErr> {
        if moves.len() == 0 {
            return Err(MovesetValidityErr::NoMoves);
        } else if moves.len() > info.len_timelines() {
            return Err(MovesetValidityErr::TooManyMoves);
        }

        // Check the validity of the moveset (whether or not it is possible to make the moves; does not look for legality)
        // Should be O(n)

        let mut timelines_moved: HashMap<Layer, bool> =
            HashMap::with_capacity(info.len_timelines());

        for mv in moves.iter() {
            if mv.from.1.t() % 2 != (if info.active_player { 0 } else { 1 })
                || mv.to.1.t() % 2 != (if info.active_player { 0 } else { 1 })
            {
                // Opponent's board
                return Err(MovesetValidityErr::OpponentBoard(*mv));
            }

            if let Some(tl) = info.get_timeline(mv.from.1.l()) {
                if mv.from.1.t() != tl.last_board {
                    // Starting board isn't the last board
                    return Err(MovesetValidityErr::MoveNotFromLast(*mv));
                }
            } else {
                // Void
                return Err(MovesetValidityErr::MoveFromVoid(*mv));
            }

            if let Some(tl) = info.get_timeline(mv.to.1.l()) {
                if *timelines_moved.get(&mv.from.1.l()).unwrap_or(&false) {
                    // Already played
                    return Err(MovesetValidityErr::AlreadyPlayed(*mv));
                }

                if mv.to.1.t() == tl.last_board {
                    // (Possibly) non-branching jump
                    timelines_moved.insert(mv.to.1.l(), true);
                } else if mv.to.1.t() > tl.last_board || mv.to.1.t() < tl.first_board {
                    // Void
                    return Err(MovesetValidityErr::MoveToVoid(*mv));
                }
                timelines_moved.insert(mv.from.1.l(), true);
            } else {
                // Void
                return Err(MovesetValidityErr::MoveToVoid(*mv));
            }
        }

        Ok(Moveset { moves })
    }
}
