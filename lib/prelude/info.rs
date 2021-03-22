use super::*;

/**
    A structure containing the necessary pieces of information for a timeline.
**/
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimelineInfo {
    /// The index of that timeline
    pub index: Layer,
    /// Where the timeline originates from
    pub starts_from: Option<(Layer, Time)>,
    /// The time coordinate of the last board of that timeline
    pub last_board: Time,
    /// The time coordinate of the first board of that timeline
    pub first_board: Time,
}

/** A structure containing the non-board data of a game state. Contains in particular the TimelineInfo for each timeline in the game. **/
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Info {
    /// Where the present for the game is at
    pub present: Time,
    /// Which player's turn it is
    pub active_player: bool,
    /// Whether or not the number of starting timelines is even. Does not change the behavior of indices,
    /// but only the behavior of functions whose behavior is dependent on the activeness of the timelines.
    pub even_timelines: bool,
    /// The timelines within `[0; +∞[` if even_timelines is false, `[0⁺; +∞[` otherwise
    pub timelines_white: Vec<TimelineInfo>,
    /// The timelines within `]-∞; -1]` if even_timelines is false, `]-∞; 0¯]` otherwise
    pub timelines_black: Vec<TimelineInfo>,
}

impl TimelineInfo {
    /** Creates a new TimelineInfo instance. **/
    pub fn new(
        index: Layer,
        starts_from: Option<(Layer, Time)>,
        last_board: Time,
        first_board: Time,
    ) -> Self {
        TimelineInfo {
            index,
            starts_from,
            last_board,
            first_board,
        }
    }
}

// timelines_white correspond to white's timelines and timelines_black correspond to black's timelines
// on an odd variant, white's timelines include the 0-timeline
// on an even variant, black's timeline include the -0-timeline and white's the +0-timeline
impl Info {
    /** Creates a new Info instance. The various pieces of informations are derived from the different TimelineInfo given. **/
    pub fn new(
        even_timelines: bool,
        timelines_white: Vec<TimelineInfo>,
        timelines_black: Vec<TimelineInfo>,
    ) -> Self {
        if timelines_white.len() == 0 {
            panic!("Expected at least one timeline!");
        }

        let min_timeline = -(timelines_black.len() as Layer);
        let max_timeline = timelines_white.len() as Layer - 1;
        let timeline_width =
            max_timeline.min(-min_timeline - (if even_timelines { 1 } else { 0 })) as usize + 2;
        let mut present = timelines_white[0].last_board;

        for tl in timelines_white.iter().take(timeline_width) {
            if tl.last_board < present {
                present = tl.last_board;
            }
        }

        for tl in timelines_black
            .iter()
            .take(timeline_width - (if even_timelines { 0 } else { 1 }))
        {
            if tl.last_board < present {
                present = tl.last_board;
            }
        }

        let active_player = present % 2 == 0;

        Info {
            present,
            active_player,
            even_timelines,
            timelines_white,
            timelines_black,
        }
    }

    /** Returns a reference to the timeline with as Layer coordinate `l` **/
    #[inline]
    pub fn get_timeline(&self, l: Layer) -> Option<&TimelineInfo> {
        if l < 0 {
            self.timelines_black.get(-l as usize - 1)
        } else {
            self.timelines_white.get(l as usize)
        }
    }

    /** Returns a mutable reference to the timeline with as Layer coordinate `l` **/
    #[inline]
    pub fn get_timeline_mut(&mut self, l: Layer) -> Option<&mut TimelineInfo> {
        if l < 0 {
            self.timelines_black.get_mut(-l as usize - 1)
        } else {
            self.timelines_white.get_mut(l as usize)
        }
    }

    /** Returns whether or not the timeline with as Layer coordinate `l` is active **/
    #[inline]
    pub fn is_active(&self, l: Layer) -> bool {
        let timeline_width = self
            .max_timeline()
            .min(-self.min_timeline() - (if self.even_timelines { 1 } else { 0 }))
            + 1;
        if l < 0 {
            if self.even_timelines {
                -l <= timeline_width + 1
            } else {
                -l <= timeline_width
            }
        } else {
            l <= timeline_width
        }
    }

    /** Returns the total number of timelines that there are in the game **/
    #[inline]
    pub fn len_timelines(&self) -> usize {
        self.timelines_black.len() + self.timelines_white.len()
    }

    /** Returns the timeline with the lowest layer coordinate. Corresponds to `min{l ∈ ℤ | timeline 'l' exists}`. **/
    #[inline]
    pub fn min_timeline(&self) -> Layer {
        -(self.timelines_black.len() as Layer)
    }

    /** Returns the timeline with the greatest layer coordinate. Corresponds to `max{l ∈ ℤ | timeline 'l' exists}`. **/
    #[inline]
    pub fn max_timeline(&self) -> Layer {
        self.timelines_white.len() as Layer - 1
    }

    /** Recalculates the present **/
    pub fn recalculate_present(&mut self) -> Time {
        let timeline_width =
            self.max_timeline()
                .min(-self.min_timeline() - (self.even_timelines as Layer)) as usize
                + 2;
        let mut present = self.timelines_white[0].last_board;

        for tl in self.timelines_white.iter().take(timeline_width) {
            if tl.last_board < present {
                present = tl.last_board;
            }
        }

        for tl in self
            .timelines_black
            .iter()
            .take(timeline_width - (!self.even_timelines) as usize)
        {
            if tl.last_board < present {
                present = tl.last_board;
            }
        }

        self.present = present;
        self.active_player = present % 2 == 0;

        present
    }

    /// Returns the number of active timelines that the player with color `white` can make.
    /// Returns 0 if they cannot make any new active timeline.
    pub fn timeline_advantage(&self, white: bool) -> usize {
        let n_timelines_white = self.timelines_white.len() - 1;
        let n_timelines_black =
            self.timelines_black.len() - if self.even_timelines { 1 } else { 0 };
        if white {
            if n_timelines_white > n_timelines_black {
                0
            } else {
                n_timelines_black + 1 - n_timelines_white
            }
        } else {
            if n_timelines_black > n_timelines_white {
                0
            } else {
                n_timelines_white + 1 - n_timelines_black
            }
        }
    }

    /// Returns the number of inactive timelines that the player with color `white` created.
    /// Returns 0 if they didn't create any inactive timeline.
    pub fn timeline_debt(&self, white: bool) -> usize {
        let n_timelines_white = self.timelines_white.len() - 1;
        let n_timelines_black =
            self.timelines_black.len() - if self.even_timelines { 1 } else { 0 };
        if white {
            if n_timelines_white <= n_timelines_black + 1 {
                0
            } else {
                n_timelines_white - 1 - n_timelines_black
            }
        } else {
            if n_timelines_black <= n_timelines_white + 1 {
                0
            } else {
                n_timelines_black - 1 - n_timelines_white
            }
        }
    }
}
