use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineAdvantage {
    pub timeline_advantage: Eval,
}

impl TimelineAdvantage {
    pub fn timeline_advantage(mut self, value: Eval) -> Self {
        self.timeline_advantage = value;
        self
    }
}

impl Default for TimelineAdvantage {
    fn default() -> Self {
        Self {
            timeline_advantage: 9.5,
        }
    }
}

impl EvalFn for TimelineAdvantage {
    fn eval<'a>(&self, _game: &'a Game, node: &'a TreeNode) -> Option<Eval> {
        let partial_game = &node.partial_game;

        let mut sum: Eval = 0.0;
        sum += partial_game.info.timeline_advantage(true) as Eval * self.timeline_advantage;
        sum -= partial_game.info.timeline_advantage(false) as Eval * self.timeline_advantage;

        if !partial_game.info.active_player {
            sum = -sum;
        }

        Some(sum)
    }
}
