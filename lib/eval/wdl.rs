// use super::*;
use std::cmp::Ordering;

const DEFAULT_COUNT: usize = 1;

/// Win/Draw/Loss primitive
#[derive(Clone, Copy, Debug)]
pub struct WDL {
    pub win: usize,
    pub draw: usize,
    pub loss: usize,
    pub sum: usize,
}

impl WDL {
    pub fn zero() -> Self {
        Self {
            win: 0,
            draw: 0,
            loss: 0,
            sum: 0
        }
    }

    pub fn draw() -> Self {
        Self {
            win: 0,
            draw: DEFAULT_COUNT,
            loss: 0,
            sum: DEFAULT_COUNT,
        }
    }

    pub fn loss() -> Self {
        Self {
            win: 0,
            draw: 0,
            loss: DEFAULT_COUNT,
            sum: DEFAULT_COUNT,
        }
    }

    pub fn win() -> Self {
        Self {
            win: DEFAULT_COUNT,
            draw: 0,
            loss: 0,
            sum: DEFAULT_COUNT,
        }
    }

    pub fn remainder(&self) -> usize {
        self.sum - (self.win + self.draw + self.loss)
    }
}

impl std::ops::Neg for WDL {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            win: self.loss,
            draw: self.draw,
            loss: self.win,
            sum: self.sum,
        }
    }
}

impl PartialOrd for WDL {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.loss * other.sum).partial_cmp(&(other.loss * self.sum)) {
            Some(Ordering::Less) => {
                Some(Ordering::Greater)
            }
            Some(Ordering::Greater) => {
                Some(Ordering::Less)
            }
            Some(Ordering::Equal) => {
                (self.win * other.sum).partial_cmp(&(other.win * self.sum))
            }
            None => None
        }
    }
}

impl PartialEq for WDL {
    fn eq(&self, other: &Self) -> bool {
        // TODO: replace == with approximations?
        self.loss * other.sum == other.loss * self.sum
        && self.win * other.sum == other.win * self.sum
    }
}
