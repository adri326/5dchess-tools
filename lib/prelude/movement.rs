use super::*;
use colored::*;
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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
        let board = partial_game.get_board_with_game(game, from.non_physical())?;
        let from: (Piece, Coords) = (partial_game.get_with_game(game, from).piece()?, from);
        let to: (Option<Piece>, Coords) = (partial_game.get_with_game(game, to).piece().into(), to);

        if from.0.white != board.white() {
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

    #[inline]
    pub fn is_jump(&self) -> bool {
        (self.from.1).0 != (self.to.1).0 || (self.from.1).1 != (self.to.1).1
    }

    pub fn generate_partial_game<'a, 'b, B>(
        &self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
        new_partial_game: &'b mut PartialGame<'a, B>
    ) -> Option<()>
    where
        'a: 'b,
        B: Clone + AsRef<Board> + From<(Board, &'a Game, &'a PartialGame<'a, B>)> + 'a
    {
        match self.kind {
            MoveKind::Normal | MoveKind::Promotion => {
                if self.is_jump() {
                    let mut new_source_board: Board = partial_game
                        .get_board_with_game(game, self.from.1.non_physical())?
                        .as_ref()
                        .clone();
                    let mut new_target_board: Board = partial_game
                        .get_board_with_game(game, self.to.1.non_physical())?
                        .as_ref()
                        .clone();
                    if new_partial_game.info.get_timeline((self.to.1).0)?.last_board > (self.to.1).1 {
                        // Branching move!
                        let new_layer = if (self.to.1).1 & 1 == 0 {
                            partial_game.info.max_timeline() + 1
                        } else {
                            partial_game.info.min_timeline() - 1
                        };
                        let new_timeline = TimelineInfo::new(
                            new_layer,
                            Some(self.to.1.non_physical()),
                            (self.to.1).1 + 1,
                            (self.to.1).1 + 1,
                        );

                        // Push the new timeline
                        if (self.to.1).1 & 1 == 0 {
                            new_partial_game.info.timelines_white.push(new_timeline);
                        } else {
                            new_partial_game.info.timelines_black.push(new_timeline);
                        }

                        new_target_board.t += 1;
                        new_target_board.l = new_layer;
                        new_source_board.t += 1;
                    } else {
                        // Non-branching move
                        new_target_board.t += 1;
                        new_source_board.t += 1;
                    }

                    let from_index = (self.from.1).2 as usize + (self.from.1).3 as usize * new_source_board.width() as usize;
                    let to_index = (self.to.1).2 as usize + (self.to.1).3 as usize * new_source_board.width() as usize;

                    if self.kind == MoveKind::Promotion {
                        let white = new_source_board.pieces[from_index].is_piece_of_color(true);
                        // NOTE: a promoted piece is considered to be unmoved;
                        // this doesn't affect the base game, but could affect
                        // customized pieces. If this is a problem to you, then
                        // replace the `false` with a `true`.
                        new_target_board.pieces[to_index] = Tile::Piece(
                            Piece::new(PieceKind::Queen, white, false)
                        );
                    } else {
                        new_target_board.pieces[to_index] = set_moved(new_source_board.pieces[from_index], true);
                    }

                    new_source_board.pieces[from_index] = Tile::Blank;

                    let new_source_coords = (new_source_board.l, new_source_board.t);
                    new_source_board.en_passant = None;
                    let new_source_board = B::from((new_source_board, game, partial_game));

                    new_partial_game.boards.insert(
                        new_source_coords,
                        new_source_board
                    );
                    new_partial_game.info.get_timeline_mut((self.from.1).0)?.last_board += 1;

                    let new_target_coords = (new_target_board.l, new_target_board.t);
                    new_target_board.en_passant = None;
                    let new_target_board = B::from((new_target_board, game, partial_game));

                    new_partial_game.boards.insert(
                        new_target_coords,
                        new_target_board
                    );
                    new_partial_game.info.get_timeline_mut((self.to.1).0)?.last_board += 1;
                } else {
                    let mut new_board: Board = partial_game
                        .get_board_with_game(game, self.from.1.non_physical())?
                        .as_ref()
                        .clone();
                    new_board.t += 1;
                    let from_index = (self.from.1).2 as usize + (self.from.1).3 as usize * new_board.width() as usize;
                    let to_index = (self.to.1).2 as usize + (self.to.1).3 as usize * new_board.width() as usize;

                    if self.kind == MoveKind::Promotion {
                        let white = new_board.t & 1 == 1;
                        // NOTE: a promoted piece is considered to be unmoved;
                        // this doesn't affect the base game, but could affect
                        // customized pieces. If this is a problem to you, then
                        // replace the `false` with a `true`.
                        new_board.pieces[to_index] = Tile::Piece(
                            Piece::new(PieceKind::Queen, white, false)
                        );
                    } else {
                        new_board.pieces[to_index] = set_moved(new_board.pieces[from_index], true);
                    }

                    new_board.pieces[from_index] = Tile::Blank;

                    if self.from.0.can_kickstart() && ((self.from.1).3 as i8 - (self.to.1).3 as i8).abs() >= 2 {
                        if new_board.t & 1 == 1 {
                            new_board.en_passant = Some(((self.from.1).2, (self.from.1).3 + 1));
                        } else {
                            new_board.en_passant = Some(((self.from.1).2, (self.from.1).3 - 1));
                        }
                    }

                    let new_coords = (new_board.l, new_board.t);
                    let new_board = B::from((new_board, game, partial_game));

                    new_partial_game.boards.insert(
                        new_coords,
                        new_board
                    );
                    new_partial_game.info.get_timeline_mut((self.from.1).0)?.last_board += 1;
                }
            }
            MoveKind::EnPassant => {
                let mut new_board: Board = partial_game
                    .get_board_with_game(game, self.from.1.non_physical())?
                    .as_ref()
                    .clone();
                let (ex, ey) = new_board.en_passant?;
                new_board.en_passant = None;
                new_board.t += 1;
                let from_index = (self.from.1).2 as usize + (self.from.1).3 as usize * new_board.width() as usize;
                let to_index = (self.to.1).2 as usize + (self.to.1).3 as usize * new_board.width() as usize;
                let capture_index = ex as usize + ey as usize * new_board.width() as usize;

                if self.kind == MoveKind::Promotion {
                    let white = new_board.pieces[from_index].is_piece_of_color(true);
                    // NOTE: a promoted piece is considered to be unmoved;
                    // this doesn't affect the base game, but could affect
                    // customized pieces. If this is a problem to you, then
                    // replace the `false` with a `true`.
                    new_board.pieces[to_index] = Tile::Piece(
                        Piece::new(PieceKind::Queen, white, false)
                    );
                } else {
                    new_board.pieces[to_index] = set_moved(new_board.pieces[from_index], true);
                }

                new_board.pieces[from_index] = Tile::Blank;
                new_board.pieces[capture_index] = Tile::Blank;

                let new_coords = (new_board.l, new_board.t);
                let new_board = B::from((new_board, game, partial_game));

                new_partial_game.boards.insert(
                    new_coords,
                    new_board
                );
                new_partial_game.info.get_timeline_mut((self.from.1).0)?.last_board += 1;
            }
            _ => {
                unimplemented!();
            }
        }
        Some(())
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:?}(L{}T{}){}{} → {}(L{}T{}){}{}",
            self.from.0,
            self.from.1.l(),
            self.from.1.t(),
            write_file(self.from.1.x()),
            self.from.1.y(),
            self.to
                .0
                .map(|x| format!("{:?}", x))
                .unwrap_or("_".white().to_string()),
            self.to.1.l(),
            self.to.1.t(),
            write_file(self.to.1.x()),
            self.to.1.y(),
        )
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

    pub fn generate_partial_game<'a, B>(
        &self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>
    ) -> Option<PartialGame<'a, B>>
    where
        B: Clone + AsRef<Board> + From<(Board, &'a Game, &'a PartialGame<'a, B>)> + 'a
    {
        let mut new_partial_game = PartialGame::new(HashMap::new(), partial_game.info.clone(), None);

        for mv in self.moves.iter() {
            mv.generate_partial_game(game, partial_game, &mut new_partial_game)?;
        }

        new_partial_game.info.recalculate_present();
        new_partial_game.parent = Some(partial_game);

        Some(new_partial_game)
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
            if mv.from.1.t() & 1 == info.active_player as Time
                || mv.to.1.t() & 1 == info.active_player as Time
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

            if *timelines_moved.get(&mv.from.1.l()).unwrap_or(&false) {
                // Already played
                return Err(MovesetValidityErr::AlreadyPlayed(*mv));
            }

            if let Some(tl) = info.get_timeline(mv.to.1.l()) {
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

/// Returns the string version of the `x` coordinate as displayed in-game
pub fn write_file(x: Physical) -> char {
    [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w',
    ][x as usize]
}

#[inline]
fn set_moved(tile: Tile, moved: bool) -> Tile {
    match tile {
        Tile::Piece(mut p) => {
            p.moved = moved;
            Tile::Piece(p)
        }
        x => x,
    }
}
