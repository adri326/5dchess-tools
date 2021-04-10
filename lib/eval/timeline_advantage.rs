use super::*;

/// Scores timeline advantage and debt
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineAdvantage {
    /// The score for a player's timeline advantage (how many new active timelines they can create, default `9.5`)
    pub timeline_advantage: Eval,
    /// The score for a player's timeline debt (how many inactive timeline they have created so far, default `0.0`)
    pub timeline_debt: Eval,
}

impl TimelineAdvantage {
    pub fn timeline_advantage(mut self, value: Eval) -> Self {
        self.timeline_advantage = value;
        self
    }

    pub fn timeline_debt(mut self, value: Eval) -> Self {
        self.timeline_debt = value;
        self
    }
}

impl Default for TimelineAdvantage {
    fn default() -> Self {
        Self {
            timeline_advantage: 9.5,
            timeline_debt: 0.0,
        }
    }
}

impl EvalFn for TimelineAdvantage {
    fn eval<'a>(&self, _game: &'a Game, node: &'a TreeNode) -> Option<Eval> {
        let partial_game = &node.partial_game;

        let mut sum: Eval = 0.0;
        sum += partial_game.info.timeline_advantage(true) as Eval * self.timeline_advantage;
        sum -= partial_game.info.timeline_advantage(false) as Eval * self.timeline_advantage;
        sum += partial_game.info.timeline_debt(true) as Eval * self.timeline_advantage;
        sum -= partial_game.info.timeline_debt(false) as Eval * self.timeline_advantage;

        if !partial_game.info.active_player {
            sum = -sum;
        }

        Some(sum)
    }
}
