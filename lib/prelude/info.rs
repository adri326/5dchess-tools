use super::*;

#[derive(Debug, Clone)]
pub struct TimelineInfo {
    pub index: Layer,
    pub starts_from: Option<(Layer, Time)>,
    pub last_board: Time,
    pub first_board: Time,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub present: Time,
    pub active_player: bool,
    pub min_timeline: Layer,
    pub max_timeline: Layer,
    pub even_timelines: bool,
    pub timelines_white: Vec<TimelineInfo>,
    pub timelines_black: Vec<TimelineInfo>,
}

impl TimelineInfo {
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
            max_timeline.min(-min_timeline - (if even_timelines { 1 } else { 0 })) as usize + 1;
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
            min_timeline,
            max_timeline,
            even_timelines,
            timelines_white,
            timelines_black,
        }
    }

    pub fn get_timeline(&self, l: Layer) -> Option<&TimelineInfo> {
        if l < 0 {
            self.timelines_black.get(-l as usize - 1)
        } else {
            self.timelines_white.get(l as usize)
        }
    }

    pub fn is_active(&self, l: Layer) -> bool {
        let timeline_width = self
            .max_timeline
            .min(-self.min_timeline - (if self.even_timelines { 1 } else { 0 }))
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

    #[inline]
    pub fn len_timelines(&self) -> usize {
        self.timelines_black.len() + self.timelines_white.len()
    }

    /// Returns the number of active timelines that the player `white` can make
    /// Returns 0 if they cannot make any new active timeline
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
}
