/*
    Structures and functions related to the game's state.
*/

use json::{object::Object, JsonValue};
use std::fmt;
use std::convert::TryFrom;

/// The main structure, contains the entire state of a game
#[derive(Debug)]
pub struct Game {
    pub timelines: Vec<Timeline>,
    pub width: usize,
    pub height: usize,
    pub info: GameInfo,
}

/// Information about whose turn it is, where the present is and timeline priority
#[derive(Debug, Clone, Copy)]
pub struct GameInfo {
    pub present: isize,
    pub active_player: bool,
    pub min_timeline: f32,
    pub max_timeline: f32,
}

/// Represents an in-game timeline
#[derive(Debug)]
pub struct Timeline {
    pub index: f32,
    pub states: Vec<Board>,
    pub width: usize,
    pub height: usize,
    pub begins_at: isize,
    pub emerges_from: Option<f32>,
}

/// Represents an in-game board (be it active or not)
#[derive(Debug, Clone)]
pub struct Board {
    pub pieces: Vec<Piece>,
    pub width: usize,
    pub height: usize,
    pub l: f32, // its timeline
    pub t: isize, // its time coordinate
    pub king_w: Option<(usize, usize)>, // TODO: update if the king moves
    pub king_b: Option<(usize, usize)>,
    pub castle_w: (bool, bool),
    pub castle_b: (bool, bool),
}

/// Represents the contents of a board's square
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    Blank,

    KingW,
    QueenW,
    PawnW,
    KnightW,
    RookW,
    BishopW,
    UnicornW,
    DragonW,
    PrincessW,

    KingB,
    QueenB,
    PawnB,
    KnightB,
    RookB,
    BishopB,
    UnicornB,
    DragonB,
    PrincessB,
}

impl From<JsonValue> for Game {
    /// Calls to From<Object> for Game
    fn from(raw: JsonValue) -> Self {
        if let JsonValue::Object(obj) = raw {
            Self::from(obj)
        } else {
            panic!("Expected JSON root element to be an object.")
        }
    }
}

impl From<Object> for Game {
    /// Constructs a Game structure from a JSON Object; this object is based on what [5dchess-notation](https://github.com/adri326/5dchess-notation) outputs
    fn from(raw: Object) -> Self {
        let timelines = match raw.get("timelines") {
            Some(JsonValue::Array(tl)) => tl
                .iter()
                .map(|raw_timeline| Timeline::from(raw_timeline))
                .collect::<Vec<Timeline>>(),
            _ => panic!("Expected JSON element 'timelines' to be an array!"),
        };

        let width = raw
            .get("width")
            .expect("Expected JSON key 'width' for Game.")
            .as_usize()
            .expect("Expected JSON element 'width' for Game to be a number");
        let height = raw
            .get("height")
            .expect("Expected JSON key 'height' for Game.")
            .as_usize()
            .expect("Expected JSON element 'height' for Game to be a number");

        let active_player = raw
            .get("active_player")
            .expect("Expected JSON key 'active_player' for Game.")
            .as_bool()
            .expect("Expected JSON element 'active_player' for Game to be a bool");
        let min_timeline = timelines
            .iter()
            .map(|tl| tl.index)
            .min_by_key(|x| (*x) as isize)
            .expect("No timeline!");
        let max_timeline = timelines
            .iter()
            .map(|tl| tl.index)
            .max_by_key(|x| (*x) as isize)
            .expect("No timeline!");

        let timeline_width = ((-min_timeline).min(max_timeline) + 1.0).round();
        let active_timelines = timelines
            .iter()
            .filter(|tl| tl.index.abs() <= timeline_width);
        let present = active_timelines
            .map(|tl| tl.begins_at + (tl.states.len() as isize) - 1)
            .min()
            .expect("No timeline!");
        let mut res = Game {
            timelines,
            width,
            height,
            info: GameInfo {
                active_player,
                present,
                min_timeline,
                max_timeline,
            },
        };

        populate_castling_rights(&mut res);

        res
    }
}

impl From<&JsonValue> for Timeline {
    // Calls to From<&Object> for Timeline
    fn from(raw: &JsonValue) -> Self {
        if let JsonValue::Object(obj) = raw {
            Self::from(obj)
        } else {
            panic!("Expected JSON Timeline to be an object.")
        }
    }
}

impl From<&Object> for Timeline {
    /// Constructs a Timeline from a JSON object; refer to From<Object> for Game
    fn from(raw: &Object) -> Self {
        let index = raw
            .get("index")
            .expect("Expected JSON to contain key 'index' for Timeline.")
            .as_f32()
            .expect("Expected JSON element 'index' for Timeline to be a number.");
        let width = raw
            .get("width")
            .expect("Expected JSON to contain key 'width' for Timeline.")
            .as_usize()
            .expect("Expected JSON element 'width' for Timeline to be a number.");
        let height = raw
            .get("height")
            .expect("Expected JSON to contain key 'height' for Timeline.")
            .as_usize()
            .expect("Expected JSON element 'height' for Timeline to be a number.");
        let begins_at = raw
            .get("begins_at")
            .expect("Expected JSON to contain key 'begins_at' for Timeline.")
            .as_isize()
            .expect("Expected JSON element 'begins_at' for Timeline to be a number.");
        let emerges_from = raw.get("emerges_from").map(|x| x.as_f32()).flatten();
        let states = match raw.get("states") {
            Some(JsonValue::Array(arr)) => {
                let mut res: Vec<Board> = Vec::new();
                let mut t: isize = 0;
                for board in arr {
                    if let JsonValue::Array(raw_pieces) = board {
                        let mut pieces: Vec<Piece> = Vec::with_capacity(width * height);
                        for raw_piece in raw_pieces {
                            pieces.push(Piece::from(
                                raw_piece.as_usize().expect("Expected piece to be a number"),
                            ));
                        }
                        res.push(Board {
                            pieces,
                            width,
                            height,
                            l: index,
                            t: t + begins_at,
                            king_w: None,
                            king_b: None,
                            castle_w: (false, false),
                            castle_b: (false, false),
                        });
                        t += 1;
                    } else {
                        panic!("Expected JSON State to be an array.")
                    }
                }
                res
            }
            _ => panic!("Expected JSON element 'states' to be an array."),
        };
        Timeline {
            index,
            states,
            width,
            height,
            begins_at,
            emerges_from,
        }
    }
}

impl From<usize> for Piece {
    /// Converts 5dchess-notation piece indices into `Piece`s.
    fn from(raw: usize) -> Self {
        match raw {
            0 => Piece::Blank,
            1 => Piece::PawnW,
            2 => Piece::KnightW,
            3 => Piece::BishopW,
            4 => Piece::RookW,
            5 => Piece::QueenW,
            6 => Piece::KingW,
            7 => Piece::UnicornW,
            8 => Piece::DragonW,
            9 => Piece::PrincessW,
            11 => Piece::PawnB,
            12 => Piece::KnightB,
            13 => Piece::BishopB,
            14 => Piece::RookB,
            15 => Piece::QueenB,
            16 => Piece::KingB,
            17 => Piece::UnicornB,
            18 => Piece::DragonB,
            19 => Piece::PrincessB,
            _ => panic!("Invalid piece: {}", raw),
        }
    }
}

impl From<Piece> for usize {
    /// Convert `Piece`s into 5dchess-notation indices
    fn from(raw: Piece) -> usize {
        match raw {
            Piece::Blank => 0,
            Piece::PawnW => 1,
            Piece::KnightW => 2,
            Piece::BishopW => 3,
            Piece::RookW => 4,
            Piece::QueenW => 5,
            Piece::KingW => 6,
            Piece::UnicornW => 7,
            Piece::DragonW => 8,
            Piece::PrincessW => 9,
            Piece::PawnB => 11,
            Piece::KnightB => 12,
            Piece::BishopB => 13,
            Piece::RookB => 14,
            Piece::QueenB => 15,
            Piece::KingB => 16,
            Piece::UnicornB => 17,
            Piece::DragonB => 18,
            Piece::PrincessB => 19,
        }
    }
}

impl Piece {
    pub fn as_uppercase(&self) -> &'static str {
        match &self {
            Piece::Blank => " ",
            Piece::KingW | Piece::KingB => "K",
            Piece::QueenW | Piece::QueenB => "Q",
            Piece::PawnW | Piece::PawnB => "P",
            Piece::BishopW | Piece::BishopB => "B",
            Piece::KnightW | Piece::KnightB => "N",
            Piece::RookW | Piece::RookB => "R",
            Piece::UnicornW | Piece::UnicornB => "U",
            Piece::DragonW | Piece::DragonB => "D",
            Piece::PrincessW | Piece::PrincessB => "S",
        }
    }

    /// Returns whether or not that `Piece` is `Piece::Blank`
    #[inline]
    pub fn is_blank(&self) -> bool {
        match &self {
            Piece::Blank => true,
            _ => false,
        }
    }

    /// Returns whether or not that `Piece` is a `Piece::*W`. Returns `false` if it is blank
    #[inline]
    pub fn is_white(&self) -> bool {
        match &self {
            Piece::PawnW
            | Piece::KnightW
            | Piece::BishopW
            | Piece::RookW
            | Piece::QueenW
            | Piece::KingW
            | Piece::UnicornW
            | Piece::DragonW => true,
            _ => false,
        }
    }

    /// Returns whether or not that `Piece` is a `Piece::*B`. Returns `false` if it is blank
    #[inline]
    pub fn is_black(&self) -> bool {
        match &self {
            Piece::PawnB
            | Piece::KnightB
            | Piece::BishopB
            | Piece::RookB
            | Piece::QueenB
            | Piece::KingB
            | Piece::UnicornB
            | Piece::DragonB => true,
            _ => false,
        }
    }

    /// Returns whether or not that `Piece` is a `Piece::King*`
    #[inline]
    pub fn is_king(&self) -> bool {
        match &self {
            Piece::KingW | Piece::KingB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Queen*`
    #[inline]
    pub fn is_queen(&self) -> bool {
        match &self {
            Piece::QueenW | Piece::QueenB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Pawn*`
    #[inline]
    pub fn is_pawn(&self) -> bool {
        match &self {
            Piece::PawnW | Piece::PawnB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Knight*`
    #[inline]
    pub fn is_knight(&self) -> bool {
        match &self {
            Piece::KnightW | Piece::KnightB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Rook*`
    #[inline]
    pub fn is_rook(&self) -> bool {
        match &self {
            Piece::RookW | Piece::RookB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Bishop*`
    #[inline]
    pub fn is_bishop(&self) -> bool {
        match &self {
            Piece::BishopW | Piece::BishopB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Unicorn*`
    #[inline]
    pub fn is_unicorn(&self) -> bool {
        match &self {
            Piece::UnicornW | Piece::UnicornB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Dragon*`
    #[inline]
    pub fn is_dragon(&self) -> bool {
        match &self {
            Piece::DragonW | Piece::DragonB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Dragon*`
    #[inline]
    pub fn is_princess(&self) -> bool {
        match &self {
            Piece::PrincessW | Piece::PrincessB => true,
            _ => false,
        }
    }

    /**
        Whether or not the piece can move by `n` tiles in any direction until it is blocked.
    **/
    #[inline]
    pub fn slides(&self) -> bool {
        match &self {
            Piece::BishopW
            | Piece::RookW
            | Piece::QueenW
            | Piece::UnicornW
            | Piece::DragonW
            | Piece::PrincessW
            | Piece::BishopB
            | Piece::RookB
            | Piece::QueenB
            | Piece::UnicornB
            | Piece::DragonB
            | Piece::PrincessB => true,
            _ => false,
        }
    }

    /// Whether or not the target piece belongs to the opponent
    #[inline]
    pub fn is_opponent_piece(&self, active_player: bool) -> bool {
        if active_player {
            self.is_black()
        } else {
            self.is_white()
        }
    }

    /// Whether or not the target piece belongs to themselves
    #[inline]
    pub fn is_own_piece(&self, active_player: bool) -> bool {
        if active_player {
            self.is_white()
        } else {
            self.is_black()
        }
    }

    /// Whether or not a piece could theoretically move to that square (by moving there or taking an opponent's piece)
    #[inline]
    pub fn is_takable_piece(&self, active_player: bool) -> bool {
        self.is_blank()
            || if active_player {
                self.is_black()
            } else {
                self.is_white()
            }
    }
}

impl Game {
    /// Returns whether or not the starting boards contain `+0L`/`-0L`
    pub fn even_initial_timelines(&self) -> bool {
        self.timelines
            .iter()
            .any(|tl| tl.index == 0.5 || tl.index == -0.5)
    }

    /// Returns the `l`-th timeline
    pub fn get_timeline<'a>(&'a self, l: f32) -> Option<&'a Timeline> {
        for timeline in self.timelines.iter() {
            if timeline.index == l {
                return Some(timeline);
            }
        }
        None
    }

    /// Returns a mutable reference to the `l`-th timeline
    pub fn get_timeline_mut<'a>(&'a mut self, l: f32) -> Option<&'a mut Timeline> {
        for timeline in self.timelines.iter_mut() {
            if timeline.index == l {
                return Some(timeline);
            }
        }
        None
    }

    /// Returns the `(l, t)` board, None if not found
    pub fn get_board<'a>(&'a self, l: f32, t: isize) -> Option<&'a Board> {
        self.get_timeline(l).map(|tl| tl.get_board(t)).flatten()
    }

    /// Returns the `(l, t)` board, panics if not found
    pub fn get_board_unsafe<'a>(&'a self, l: f32, t: isize) -> &'a Board {
        self.get_timeline(l).expect("Couldn't find timeline!").get_board_unsafe(t)
    }

    /// Returns a mutable reference to the `(l, t)` board, None if not found
    pub fn get_board_mut<'a>(&'a mut self, l: f32, t: isize) -> Option<&'a mut Board> {
        self.get_timeline_mut(l)
            .map(|tl| tl.get_board_mut(t))
            .flatten()
    }

    /// Returns a mutable reference to the `(l, t)` board, panics if not found
    pub fn get_board_mut_unsafe<'a>(&'a mut self, l: f32, t: isize) -> &'a mut Board {
        self.get_timeline_mut(l)
            .expect("Couldn't find timeline!")
            .get_board_mut_unsafe(t)
    }

    /// Returns the last board in the `l`-th timeline, None if not found
    pub fn get_last_board<'a>(&'a self, l: f32) -> Option<&'a Board> {
        self.get_timeline(l).map(|tl| tl.get_last_board()).flatten()
    }

    /// Returns the `(l, t, x, y)` piece, None if not found
    pub fn get<'a>(&'a self, l: f32, t: isize, x: usize, y: usize) -> Option<Piece> {
        self.get_timeline(l).map(|tl| tl.get(t, x, y)).flatten()
    }

    /// Returns the `(l, t, x, y)` piece, panics if not found
    pub fn get_unsafe<'a>(&'a self, l: f32, t: isize, x: usize, y: usize) -> Piece {
        self.get_timeline(l).expect("Couldn't find timeline!").get_unsafe(t, x, y)
    }

    /** Appends a set of boards to the current game structure; currently only supports appending one board to every timeline.
        This is usually used together with `Move::generate_vboards`:

        ```
        let mut game: Game;
        let mv: Move;

        // Assign to `mv` and `game`

        let boards = mv.generate_vboards(&game, &game.info, &vec![], &vec![]);
        game.commit_moves(boards);
        ```
    **/
    pub fn commit_moves(&mut self, mut boards: Vec<Board>) {
        boards.sort_by_key(|b| b.t);
        boards.reverse();
        for b in boards.into_iter() {
            if let Some(tl) = self.get_timeline_mut(b.l) {
                if tl.get_board(b.t).is_none() {
                    tl.states.push(b)
                } else {
                    println!("{:?}", tl.get_board(b.t));
                    println!("{:?}", b);
                    panic!("Board already there: {}/{}", b.l, b.t);
                }
            } else {
                self.timelines.push(Timeline {
                    index: b.l,
                    begins_at: b.t,
                    width: self.width,
                    height: self.height,
                    emerges_from: None,
                    states: vec![b],
                });
            }
        }
        self.info.active_player = !self.info.active_player;
    }
}

impl Timeline {
    /// Returns the last board in this timeline, None if the timeline is still empty (which it shouldn't if created by normal means)
    pub fn get_last_board<'a>(&'a self) -> Option<&'a Board> {
        self.states.get(self.states.len() - 1)
    }

    /// Returns the board at `t` in this timeline (this will be different from the `t`-th board if the timeline is synthetic), None if not found
    pub fn get_board<'a>(&'a self, t: isize) -> Option<&'a Board> {
        if t < self.begins_at {
            None
        } else {
            self.states.get(usize::try_from(t - self.begins_at).ok()?)
        }
    }

    /// Returns the board at `t` in this timeline; panics if the board does not exist
    pub fn get_board_unsafe<'a>(&'a self, t: isize) -> &'a Board {
        &self.states[(t - self.begins_at) as usize]
    }

    /// Returns a mutable reference to the board at `t` in this timeline, None if not found
    pub fn get_board_mut<'a>(&'a mut self, t: isize) -> Option<&'a mut Board> {
        if t < self.begins_at {
            None
        } else {
            self.states.get_mut(usize::try_from(t - self.begins_at).ok()?)
        }
    }

    /// Returns a mutable reference to the board at `t` in this timeline, panics if the board does not exist
    pub fn get_board_mut_unsafe<'a>(&'a mut self, t: isize) -> &'a mut Board {
        &mut self.states[(t - self.begins_at) as usize]
    }

    /// Returns the piece at `(t, x, y)` in this timeline, None if not found
    pub fn get<'a>(&'a self, t: isize, x: usize, y: usize) -> Option<Piece> {
        self.get_board(t).map(|board| board.get(x, y)).flatten()
    }

    /// Returns the piece at `(t, x, y)` in this timeline, panics if the square does not exist. UB if that board's size is not equal to the timeline's own size
    pub fn get_unsafe<'a>(&'a self, t: isize, x: usize, y: usize) -> Piece {
        self.states[(t - self.begins_at) as usize].pieces[x + self.width * y]
    }
}

impl Board {
    /// Returns the piece at `(x, y)`, None if not found
    pub fn get(&self, x: usize, y: usize) -> Option<Piece> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.pieces.get(x + y * self.width).copied()
        }
    }

    /// Returns the piece at `(x, y)`, panics if not found
    pub fn get_unsafe(&self, x: usize, y: usize) -> Piece {
        self.pieces[x + y * self.width]
    }

    /// Sets the piece at `(x, y)`, returns `Ok` on success and `Err` if the square does not exist
    pub fn set(&mut self, x: usize, y: usize, piece: Piece) -> Result<(), ()> {
        if x >= self.width || y >= self.height {
            Err(())
        } else {
            self.pieces[x + y * self.width] = piece;
            Ok(())
        }
    }

    /// Sets the piece at `(x, y)`, panics if the square does not exist
    pub fn set_unsafe(&mut self, x: usize, y: usize, piece: Piece) {
        self.pieces[x + y * self.width] = piece;
    }

    /// Returns whose player's turn it is on this board
    pub fn active_player(&self) -> bool {
        self.t % 2 == 0
    }

    /// Returns whether or not this board must be played on (does not check if it is the last board in its timeline)
    pub fn is_active(&self, info: &GameInfo) -> bool {
        self.t <= info.present
            && if self.l < 0.0 {
                self.l >= -info.max_timeline.round() - 1.0
            } else {
                self.l <= -info.min_timeline.round() + 1.0
            }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Piece::Blank => ".",
                Piece::PawnW => "P",
                Piece::KnightW => "N",
                Piece::BishopW => "B",
                Piece::RookW => "R",
                Piece::QueenW => "Q",
                Piece::KingW => "K",
                Piece::UnicornW => "U",
                Piece::DragonW => "D",
                Piece::PrincessW => "S",
                Piece::PawnB => "p",
                Piece::KnightB => "n",
                Piece::BishopB => "b",
                Piece::RookB => "r",
                Piece::QueenB => "q",
                Piece::KingB => "k",
                Piece::UnicornB => "u",
                Piece::DragonB => "d",
                Piece::PrincessB => "s",
            }
        )
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                write!(f, "{}", self.pieces[x + y * self.height])?;
            }
            if y > 0 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

/// Applies a function on the `(l, t)` board and on all of its predecessors, stops if the function returns false
pub fn bubble_up<'a, F>(game: &'a Game, mut l: f32, mut t: isize, mut f: F)
where
    F: FnMut(&'a Board) -> bool,
{
    while let Some(board) = game.get_board(l, t) {
        if !f(board) {
            break;
        }
        t -= 1;
        let tl = game.get_timeline(l).unwrap();
        if t < tl.begins_at {
            if let Some(l2) = tl.emerges_from {
                l = l2;
            }
        }
    }
}

/** Applies a function on the `(l, t)` board and on all of its descendants, passes on a value down each branch of boards and stops if the function returns false.

    ## Example

    ```
    bubble_down(game, 0, 0, |board, _| {
        if board.get_unsafe(0, 0).is_blank() {
            (false, ())
        } else {
            println!("Piece on board {}/{} at 0:0 is {}", board.l, board.t, board.get_unsafe(0, 0))
        }
    }, ());
    ```
**/
pub fn bubble_down<'a, F, T>(game: &'a mut Game, l: f32, mut t: isize, mut f: F, initial: T)
where
    F: FnMut(&'_ mut Board, T) -> (bool, T),
    F: Copy,
    T: Copy,
{
    let checkpoints: Vec<(f32, isize)> = game
        .timelines
        .iter()
        .filter(|tl| tl.emerges_from == Some(l))
        .map(|tl| (tl.index, tl.begins_at))
        .filter(|x| x.1 > t)
        .collect();
    let mut previous_value: T = initial;
    loop {
        match game.get_board_mut(l, t) {
            Some(b) => {
                let res = f(b, previous_value);
                if !res.0 {
                    break;
                }
                previous_value = res.1;
            }
            _ => break,
        }

        t += 1;
        checkpoints.iter().for_each(|(index, begins_at)| {
            if *begins_at == t + 1 {
                bubble_down(game, *index, t + 1, f, previous_value)
            }
        });
    }
}

/// Returns the index of the timeline "below" (whose index is lower than) the `l`-th timeline; takes in account games with an even number of starting boards
pub fn timeline_below(game: &Game, l: f32) -> f32 {
    if l == 1.0 && game.even_initial_timelines() {
        0.5
    } else if l == -0.5 {
        -1.0
    } else {
        l - 1.0
    }
}

/// Returns the index of the timeline "above" (whose index is greater than) the `l`-th timeline; takes in account games with an even number of starting boards
pub fn timeline_above(game: &Game, l: f32) -> f32 {
    if l == -1.0 && game.even_initial_timelines() {
        -0.5
    } else if l == 0.5 {
        1.0
    } else {
        l + 1.0
    }
}

/// Returns the index of the timeline that is `dl` timelines above/below the `l`-th timeline; takes in account games with an even number of starting boards
pub fn shift_timeline(game: &Game, mut l: f32, dl: isize) -> f32 {
    if game.even_initial_timelines() {
        if l == -0.5 {
            l = 0.0;
        } else if l == 0.5 {
            l = 1.0;
        } else if l > 0.0 {
            l += 1.0;
        }
        l += dl as f32;
        if l == 0.0 {
            -0.5
        } else if l == 1.0 {
            0.5
        } else if l > 1.0 {
            l - 1.0
        } else {
            l
        }
    } else {
        l + dl as f32
    }
}

/// Returns the string version out the timeline index as displayed in-game; does not prepend a `+` if `l >= 1`
pub fn write_timeline(l: f32) -> String {
    if l == -0.5 {
        String::from("-0")
    } else if l == 0.5 {
        String::from("+0")
    } else {
        (l as isize).to_string()
    }
}

/// Returns the string version of the `x` coordinate as displayed in-game
pub fn write_file(x: usize) -> char {
    [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w',
    ][x]
}

/**
    Populates the castling rights of every board in the game; does so by induction (uses the `bubble_down` function)
**/
pub fn populate_castling_rights(game: &mut Game) {
    // I apologize to code readers for the visual density of this function

    let width = game.width;
    let height = game.height;
    let timeline_indices: Vec<f32> = game
        .timelines
        .iter()
        .filter(|tl| tl.begins_at == 0)
        .map(|tl| tl.index.clone())
        .collect();

    for l in timeline_indices {
        // Extract white and black's position
        let (king_w, king_b) = {
            let board = game.get_board(l, 0).unwrap();
            let kings_w: Vec<(usize, Piece)> = board
                .pieces
                .iter()
                .copied()
                .enumerate()
                .filter(|(_i, p)| *p == Piece::KingW)
                .collect();
            let kings_b: Vec<(usize, Piece)> = board
                .pieces
                .iter()
                .copied()
                .enumerate()
                .filter(|(_i, p)| *p == Piece::KingB)
                .collect();
            if kings_w.len() != 1 && kings_b.len() != 1 {
                continue;
            }
            (
                kings_w.get(0).map(|k| (k.0 % width, k.0 / width)),
                kings_b.get(0).map(|k| (k.0 % width, k.0 / width)),
            )
        };

        // Get rook positions
        let (rook_w1, rook_w2, rook_b1, rook_b2) = {
            let board = game.get_board(l, 0).unwrap();

            let rooks_w: Vec<(usize, Piece)> = board
                .pieces
                .iter()
                .copied()
                .enumerate()
                .filter(|(i, p)| {
                    *p == Piece::RookW && *i / width == king_w.map(|k| k.1).unwrap_or(height)
                })
                .collect();
            let rooks_b: Vec<(usize, Piece)> = board
                .pieces
                .iter()
                .copied()
                .enumerate()
                .filter(|(i, p)| {
                    *p == Piece::RookB && *i / width == king_b.map(|k| k.1).unwrap_or(height)
                })
                .collect();

            let rook_w_left = rooks_w
                .iter()
                .filter(|(i, _p)| i % width < king_w.map(|k| k.0).unwrap_or(0))
                .max_by_key(|(i, _p)| i % width);
            let rook_w_right = rooks_w
                .iter()
                .filter(|(i, _p)| i % width > king_w.map(|k| k.0).unwrap_or(width))
                .min_by_key(|(i, _p)| i % width);
            let rook_b_left = rooks_b
                .iter()
                .filter(|(i, _p)| i % width < king_b.map(|k| k.0).unwrap_or(0))
                .max_by_key(|(i, _p)| i % width);
            let rook_b_right = rooks_b
                .iter()
                .filter(|(i, _p)| i % width > king_b.map(|k| k.0).unwrap_or(width))
                .min_by_key(|(i, _p)| i % width);

            (
                rook_w_left.map(|(i, _p)| (i % width, i / width)),
                rook_w_right.map(|(i, _p)| (i % width, i / width)),
                rook_b_left.map(|(i, _p)| (i % width, i / width)),
                rook_b_right.map(|(i, _p)| (i % width, i / width)),
            )
        };

        // Bubble down!
        bubble_down(
            game,
            l,
            0,
            |board, mut last_state| {
                if last_state.0 {
                    last_state.0 = rook_w1
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p == Piece::RookW)
                        .unwrap();
                }
                if last_state.1 {
                    last_state.1 = rook_w2
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p == Piece::RookW)
                        .unwrap();
                }
                if last_state.2 {
                    last_state.2 = rook_b1
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p == Piece::RookB)
                        .unwrap();
                }
                if last_state.3 {
                    last_state.3 = rook_b2
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p == Piece::RookB)
                        .unwrap();
                }
                if last_state.0 || last_state.1 {
                    if king_w
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p != Piece::KingW)
                        .unwrap()
                    {
                        last_state.0 = false;
                        last_state.1 = false;
                    }
                }
                if last_state.2 || last_state.3 {
                    if king_b
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p != Piece::KingB)
                        .unwrap()
                    {
                        last_state.2 = false;
                        last_state.3 = false;
                    }
                }

                board.king_w = king_w;
                board.king_b = king_b;
                board.castle_w = (last_state.0, last_state.1);
                board.castle_b = (last_state.2, last_state.3);

                (true, last_state)
            },
            (
                rook_w1.is_some() && king_w.is_some(),
                rook_w2.is_some() && king_w.is_some(),
                rook_b1.is_some() && king_b.is_some(),
                rook_b2.is_some() && king_b.is_some(),
            ),
        );
    }
}
