use super::*;
use colored::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};

/** Represents a move's kind (regular move, castling move, etc.) **/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveKind {
    Normal,
    Castle,
    EnPassant,
    Promotion,
}

/** Used by Move::generate_partial_game(...). This has no effect if the move is a physical move. **/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialGameGenKind {
    /** Whether to generate both the new target and source boards **/
    Both,
    /** Whether to only generate the new source board **/
    Source,
    /** Whether to only generate the new target board **/
    Target,
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
    pub fn new(game: &Game, partial_game: &PartialGame, from: Coords, to: Coords) -> Option<Self> {
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
            && ((to.1).3 == 0 && (from.1).3 != 0
                || (to.1).3 == game.height - 1 && (from.1).3 != game.height - 1)
        {
            kind = MoveKind::Promotion;
        } else if from.0.can_castle() && ((from.1).2 == (to.1).2 + 2 || (from.1).2 + 2 == (to.1).2)
        {
            kind = MoveKind::Castle;
        }

        Some(Self { from, to, kind })
    }

    #[inline]
    pub fn from_raw(from: (Piece, Coords), to: (Option<Piece>, Coords), kind: MoveKind) -> Self {
        Self { from, to, kind }
    }

    #[inline]
    pub fn captures(&self) -> bool {
        self.to.0.is_some()
    }

    #[inline]
    pub fn is_jump(&self) -> bool {
        (self.from.1).0 != (self.to.1).0 || (self.from.1).1 != (self.to.1).1
    }

    /**
        Generates boards and updates a mutable PartialGame, given a single move.
        You should use `Moveset::generate_partial_game` if you're using Movesets!

        Should you need to use this function instead of `Moveset::generate_partial_game`, then know that:
        - if `kind == PartialGameGenKind::Both`, this function will run normally
        - if `kind == PartialGameGenKind::Source`, this function will not generate the new target board when a non-physical move is encountered
        - if `kind == PartialGameGenKind::Target`, this function will not generate the new source board when a non-physical move is encountered
        - the present in `new_partial_game` will not be changed; moreover, this function does not check if the new source or target board are already
            in `new_partial_game` and will overwrite them.
    **/
    pub fn generate_partial_game<'a, 'b>(
        &self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a>,
        new_partial_game: &'b mut PartialGame<'a>,
        kind: PartialGameGenKind,
    ) -> Option<()> {
        let white = (self.from.1).1 & 1 == 0;
        match self.kind {
            MoveKind::Normal | MoveKind::Promotion => {
                if self.is_jump() {
                    // If the move is a jump...
                    // Clone the boards
                    let mut new_source_board: Board = partial_game
                        .get_board_with_game(game, self.from.1.non_physical())?
                        .clone();
                    let mut new_target_board: Board = partial_game
                        .get_board_with_game(game, self.to.1.non_physical())?
                        .clone();

                    // Handle branching
                    if kind != PartialGameGenKind::Source {
                        if new_partial_game
                            .info
                            .get_timeline((self.to.1).0)?
                            .last_board
                            > (self.to.1).1
                        {
                            // Branching move!
                            let new_layer = if white {
                                new_partial_game.info.max_timeline() + 1
                            } else {
                                new_partial_game.info.min_timeline() - 1
                            };

                            // Generate a new timeline info
                            let new_timeline = TimelineInfo::new(
                                new_layer,
                                Some(self.to.1.non_physical()),
                                (self.to.1).1 + 1,
                                (self.to.1).1 + 1,
                            );

                            // Push the new timeline info
                            if white {
                                new_partial_game.info.timelines_white.push(new_timeline);
                            } else {
                                new_partial_game.info.timelines_black.push(new_timeline);
                            }

                            new_target_board.t += 1;
                            new_target_board.l = new_layer;
                        } else {
                            // Non-branching move
                            new_partial_game
                                .info
                                .get_timeline_mut((self.to.1).0)?
                                .last_board += 1;
                            new_target_board.t += 1;
                        }
                    }

                    new_source_board.t += 1;

                    // Calculate indices
                    let from_index = (self.from.1).2 as usize
                        + (self.from.1).3 as usize * new_source_board.width() as usize;
                    let to_index = (self.to.1).2 as usize
                        + (self.to.1).3 as usize * new_source_board.width() as usize;

                    // Update target board pieces and handle promotion
                    if self.kind == MoveKind::Promotion {
                        // NOTE: a promoted piece is considered to be unmoved;
                        // this doesn't affect the base game, but could affect
                        // customized pieces. If this is a problem to you, then
                        // replace the `false` with a `true`.
                        new_target_board.pieces[to_index] =
                            Tile::Piece(Piece::new(PieceKind::Queen, white, false));
                    } else {
                        new_target_board.pieces[to_index] =
                            set_moved(new_source_board.pieces[from_index], true);
                    }

                    new_source_board.pieces[from_index] = Tile::Blank;

                    // Insert source board
                    if kind != PartialGameGenKind::Target {
                        let new_source_coords = (new_source_board.l, new_source_board.t);
                        new_source_board.en_passant = None;

                        new_partial_game
                            .boards
                            .insert(new_source_coords, new_source_board);
                        new_partial_game
                            .info
                            .get_timeline_mut((self.from.1).0)?
                            .last_board += 1;
                    }

                    // Insert target board
                    if kind != PartialGameGenKind::Source {
                        let new_target_coords = (new_target_board.l, new_target_board.t);
                        new_target_board.en_passant = None;

                        new_partial_game
                            .boards
                            .insert(new_target_coords, new_target_board);
                    }
                } else {
                    // If the move isn't a jump...
                    // Clone the board and update its metadata
                    let mut new_board: Board = partial_game
                        .get_board_with_game(game, self.from.1.non_physical())?
                        .clone();
                    new_board.t += 1;

                    // Calculate the indices
                    let from_index = (self.from.1).2 as usize
                        + (self.from.1).3 as usize * new_board.width() as usize;
                    let to_index = (self.to.1).2 as usize
                        + (self.to.1).3 as usize * new_board.width() as usize;

                    // Piece promotion
                    if self.kind == MoveKind::Promotion {
                        // NOTE: a promoted piece is considered to be unmoved;
                        // this doesn't affect the base game, but could affect
                        // customized pieces. If this is a problem to you, then
                        // replace the `false` with a `true`.
                        new_board.pieces[to_index] =
                            Tile::Piece(Piece::new(PieceKind::Queen, white, false));
                    } else {
                        new_board.pieces[to_index] = set_moved(new_board.pieces[from_index], true);
                    }

                    // Update pieces
                    new_board.pieces[from_index] = Tile::Blank;

                    // Set en passant information
                    if self.from.0.can_kickstart()
                        && ((self.from.1).3 as i8 - (self.to.1).3 as i8).abs() >= 2
                    {
                        if new_board.t & 1 == 1 {
                            new_board.en_passant = Some(((self.from.1).2, (self.from.1).3 + 1));
                        } else {
                            new_board.en_passant = Some(((self.from.1).2, (self.from.1).3 - 1));
                        }
                    } else {
                        new_board.en_passant = None;
                    }

                    // Insert board and update timeline info
                    let new_coords = (new_board.l, new_board.t);

                    new_partial_game.boards.insert(new_coords, new_board);
                    new_partial_game
                        .info
                        .get_timeline_mut((self.from.1).0)?
                        .last_board += 1;
                }
            }
            MoveKind::EnPassant => {
                // Clone the board and update its metadata
                let mut new_board: Board = partial_game
                    .get_board_with_game(game, self.from.1.non_physical())?
                    .clone();
                let (ex, ey) = new_board.en_passant?;
                new_board.en_passant = None;
                new_board.t += 1;

                // Generate the indices
                let from_index = (self.from.1).2 as usize
                    + (self.from.1).3 as usize * new_board.width() as usize;
                let to_index =
                    (self.to.1).2 as usize + (self.to.1).3 as usize * new_board.width() as usize;
                let capture_index = ex as usize + ey as usize * new_board.width() as usize;

                // Replace pieces
                new_board.pieces[to_index] = set_moved(new_board.pieces[from_index], true);
                new_board.pieces[from_index] = Tile::Blank;
                new_board.pieces[capture_index] = Tile::Blank;

                // Insert board and update timeline info
                let new_coords = (new_board.l, new_board.t);

                new_partial_game.boards.insert(new_coords, new_board);
                new_partial_game
                    .info
                    .get_timeline_mut((self.from.1).0)?
                    .last_board += 1;
            }
            MoveKind::Castle => {
                // Clone the board and update its metadata
                let white = (self.from.1).1 & 1 == 0;
                let mut new_board: Board = partial_game
                    .get_board_with_game(game, self.from.1.non_physical())?
                    .clone();
                new_board.en_passant = None;
                new_board.t += 1;

                // Find out the castling direction and rook position
                let direction = (self.from.1).2 > (self.to.1).2;
                let rook_position = if direction {
                    let mut x = (self.from.1).2;
                    let y = (self.from.1).3;
                    loop {
                        x -= 1;
                        if let Tile::Piece(p) = new_board.get((x, y)) {
                            if p.white == white && p.can_castle_to() && !p.moved {
                                break Some((x, y));
                            } else {
                                break None;
                            }
                        }
                        if x <= 0 {
                            break None;
                        }
                    }
                } else {
                    let mut x = (self.from.1).2;
                    let y = (self.from.1).3;
                    loop {
                        x += 1;
                        if let Tile::Piece(p) = new_board.get((x, y)) {
                            if p.white == white && p.can_castle_to() && !p.moved {
                                break Some((x, y));
                            } else {
                                break None;
                            }
                        }
                        if x >= new_board.width() - 1 {
                            break None;
                        }
                    }
                };
                let rook_position = rook_position?;

                // Calculate the indices
                let rook_index = rook_position.0 as usize
                    + rook_position.1 as usize * new_board.width() as usize;
                let from_index = (self.from.1).2 as usize
                    + (self.from.1).3 as usize * new_board.width() as usize;
                let to_index =
                    (self.to.1).2 as usize + (self.to.1).3 as usize * new_board.width() as usize;

                // Update pieces
                new_board.pieces[to_index] = set_moved(new_board.pieces[from_index], true);
                new_board.pieces[from_index] = Tile::Blank;
                if direction {
                    new_board.pieces[from_index - 1] =
                        set_moved(new_board.pieces[rook_index], true);
                } else {
                    new_board.pieces[from_index + 1] =
                        set_moved(new_board.pieces[rook_index], true);
                }
                new_board.pieces[rook_index] = Tile::Blank;

                // Insert board and update timeline info
                let new_coords = (new_board.l, new_board.t);

                new_partial_game.boards.insert(new_coords, new_board);
                new_partial_game
                    .info
                    .get_timeline_mut((self.from.1).0)?
                    .last_board += 1;
            }
        }
        Some(())
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}(L{}T{}){}{} → {}(L{}T{}){}{}",
            self.from.0,
            self.from.1.l(),
            self.from.1.t(),
            write_file(self.from.1.x()),
            self.from.1.y() + 1,
            self.to
                .0
                .map(|x| format!("{:?}", x))
                .unwrap_or("_".white().to_string()),
            self.to.1.l(),
            self.to.1.t(),
            write_file(self.to.1.x()),
            self.to.1.y() + 1,
        )
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(L{}T{})", self.from.1.l(), self.from.1.t() / 2 + 1)?;
        if self.kind == MoveKind::Castle {
            if (self.from.1).2 > (self.to.1).2 {
                write!(f, "O-O-O")
            } else {
                write!(f, "O-O")
            }
        } else {
            write!(
                f,
                "{:?}{}{}",
                self.from.0.kind,
                write_file((self.from.1).2),
                (self.from.1).3 + 1
            )?;

            if self.to.1.non_physical() != self.from.1.non_physical() {
                // I currently have no way to verify that the move is branching or not;
                // The parser can solve this later anyways
                write!(f, ">>")?;
            }

            if self.to.0.is_some() {
                write!(f, "x")?;
            }

            if self.to.1.non_physical() != self.from.1.non_physical() {
                write!(f, "(L{}T{})", self.to.1.l(), self.to.1.t() / 2 + 1)?;
            }

            if self.kind == MoveKind::Promotion {
                write!(f, "{}{}=Q", write_file((self.to.1).2), (self.to.1).3 + 1)
            } else {
                write!(f, "{}{}", write_file((self.to.1).2), (self.to.1).3 + 1)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    #[inline]
    pub fn new(moves: Vec<Move>, info: &Info) -> Result<Moveset, MovesetValidityErr> {
        Self::try_from((moves, info))
    }

    #[inline]
    pub fn moves(&self) -> &Vec<Move> {
        &self.moves
    }

    /**
        Generates a new PartialGame from a Moveset. The new PartialGame will have its `present`
        and `active_player` updated and will contain all of the boards generated by the underlying moveset.

        Because `Moveset`s don't store the `Info` it was validated against, generating a `PartialGame`
        with a different `Info` from the one used when validating the `Moveset` will cause undefined behavior.
    **/
    pub fn generate_partial_game<'a>(
        &self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a>,
    ) -> Option<PartialGame<'a>> {
        let mut new_partial_game =
            PartialGame::new(HashMap::new(), partial_game.info.clone(), None);

        for mv in self.moves.iter() {
            mv.generate_partial_game(
                game,
                partial_game,
                &mut new_partial_game,
                PartialGameGenKind::Both,
            )?;
        }

        new_partial_game.info.recalculate_present();
        new_partial_game.parent = Some(partial_game);

        if new_partial_game.info.active_player != partial_game.info.active_player {
            Some(new_partial_game)
        } else {
            None
        }
    }
}

impl Hash for Moveset {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut moves = self.moves.clone();
        moves.sort_by(|mv_a, mv_b| (mv_a.from.1).0.partial_cmp(&(mv_b.from.1).0).unwrap());
        for mv in moves {
            mv.hash(state);
        }
    }
}

// TODO: better implementation of PartialEq
impl PartialEq for Moveset {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let mut self_moves = self.moves.clone();
        self_moves.sort_by(|mv_a, mv_b| (mv_a.from.1).0.partial_cmp(&(mv_b.from.1).0).unwrap());
        let mut other_moves = other.moves.clone();
        other_moves.sort_by(|mv_a, mv_b| (mv_a.from.1).0.partial_cmp(&(mv_b.from.1).0).unwrap());
        self_moves == other_moves
    }
}

impl Eq for Moveset {}

impl fmt::Display for Moveset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for mv in self.moves.iter() {
            write!(f, "{} ", mv)?
        }
        Ok(())
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
