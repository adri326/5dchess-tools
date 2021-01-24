use super::*;

/**
    A goal - or objective - represents a condition that branches of the tree search must fulfill.
    The verification for the fulfillement of the condition is done using the `verify` function.
    It takes as argument:
    - a reference to a `Moveset`
    - a `&Game` reference
    - a `&PartialGame` reference, which corresponds to the new `PartialGame` generated by the `Moveset`
    - a depth value, representing the depth of the tree search

    If for instance, you wish to only keep the movesets where no new timeline gets created until depth>4, you'd do:

    ```
    pub struct MyGoal {
        pub min_timeline: Layer,
        pub max_timeline: Layer,
    }

    impl<B: Clone + AsRef<Board>> Goal<B> for MyGoal {
        fn verify<'b>(
            &self,
            moveset: &'b Moveset,
            game: &'b Game,
            partial_game: &'b PartialGame<'b, B>,
            depth: usize
        ) -> Option<bool> {
            if depth > 4 {
                Some(true)
            } else {
                Some(
                    partial_game.info.min_timeline() == self.min_timeline
                    && partial_game.info.max_timeline() == self.max_timeline
                )
            }
        }
    }
    ```

    A similar goal can be found at `goals::misc::NoBranching`.

    You may use the included `or`, `and` and `not` functions to combine goals together.
**/
pub trait Goal<B>
where
    B: Clone + AsRef<Board>,
{
    /**
        Required method. Return `Some(true)` if the given moveset is valid, `Some(false)` if it is invalid and `None` if an error occured.
    **/
    fn verify<'b>(&self, moveset: &'b Moveset, game: &'b Game, partial_game: &'b PartialGame<'b, B>, depth: usize) -> Option<bool>;

    fn or<G: Goal<B>>(self, goal: G) -> OrGoal<B, Self, G>
    where
        Self: Sized
    {
        OrGoal::new(self, goal)
    }

    fn and<G: Goal<B>>(self, goal: G) -> AndGoal<B, Self, G>
    where
        Self: Sized
    {
        AndGoal::new(self, goal)
    }

    fn not(self) -> NotGoal<B, Self>
    where
        Self: Sized
    {
        NotGoal::new(self)
    }
}

/** A goal that will always return true. **/
pub struct TrueGoal;

impl<B> Goal<B> for TrueGoal
where
    B: Clone + AsRef<Board>,
{
    fn verify<'b>(&self, _moveset: &'b Moveset, _game: &'b Game, _partial_game: &'b PartialGame<'b, B>, _depth: usize) -> Option<bool> {
        Some(true)
    }
}

/** A goal that will always return false. **/
pub struct FalseGoal;

impl<B> Goal<B> for FalseGoal
where
    B: Clone + AsRef<Board>,
{
    fn verify<'b>(&self, _moveset: &'b Moveset, _game: &'b Game, _partial_game: &'b PartialGame<'b, B>, _depth: usize) -> Option<bool> {
        Some(false)
    }
}

/**
    A goal that will return true if its sub-goal returns false, representing the negation of its sub-goal.
    If the sub-goal fails (by returning `None`), it will also fail (by returning `None`).
**/
pub struct NotGoal<B, G>
where
    B: Clone + AsRef<Board>,
    G: Goal<B>,
{
    pub goal: G,
    _phantom: std::marker::PhantomData<B>,
}

impl<B, G> NotGoal<B, G>
where
    B: Clone + AsRef<Board>,
    G: Goal<B>,
{
    pub fn new(goal: G) -> Self {
        Self {
            goal,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<B, G> Goal<B> for NotGoal<B, G>
where
    B: Clone + AsRef<Board>,
    G: Goal<B>,
{
    fn verify<'b>(&self, moveset: &'b Moveset, game: &'b Game, partial_game: &'b PartialGame<'b, B>, depth: usize) -> Option<bool> {
        match self.goal.verify(moveset, game, partial_game, depth) {
            Some(x) => Some(!x),
            None => None
        }
    }
}

/**
    A goal that will return true if either of its sub-goals returns true, representing the disjunction of both goals.
**/
pub struct OrGoal<B, Left, Right>
where
    B: Clone + AsRef<Board>,
    Left: Goal<B>,
    Right: Goal<B>,
{
    pub left: Left,
    pub right: Right,
    _phantom: std::marker::PhantomData<B>,
}

impl<B, Left, Right> OrGoal<B, Left, Right>
where
    B: Clone + AsRef<Board>,
    Left: Goal<B>,
    Right: Goal<B>,
{
    pub fn new(left: Left, right: Right) -> Self {
        Self {
            left,
            right,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<B, Left, Right> Goal<B> for OrGoal<B, Left, Right>
where
    B: Clone + AsRef<Board>,
    Left: Goal<B>,
    Right: Goal<B>,
{
    fn verify<'b>(&self, moveset: &'b Moveset, game: &'b Game, partial_game: &'b PartialGame<'b, B>, depth: usize) -> Option<bool> {
        match self.left.verify(moveset, game, partial_game, depth) {
            Some(false) => self.right.verify(moveset, game, partial_game, depth),
            x => x
        }
    }
}

/**
    A goal that returns true if both sub-goals return true, representing the conjunction of both goals.
**/
pub struct AndGoal<B, Left, Right>
where
    B: Clone + AsRef<Board>,
    Left: Goal<B>,
    Right: Goal<B>,
{
    pub left: Left,
    pub right: Right,
    _phantom: std::marker::PhantomData<B>,
}

impl<B, Left, Right> AndGoal<B, Left, Right>
where
    B: Clone + AsRef<Board>,
    Left: Goal<B>,
    Right: Goal<B>,
{
    pub fn new(left: Left, right: Right) -> Self {
        Self {
            left,
            right,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<B, Left, Right> Goal<B> for AndGoal<B, Left, Right>
where
    B: Clone + AsRef<Board>,
    Left: Goal<B>,
    Right: Goal<B>,
{
    fn verify<'b>(&self, moveset: &'b Moveset, game: &'b Game, partial_game: &'b PartialGame<'b, B>, depth: usize) -> Option<bool> {
        match self.left.verify(moveset, game, partial_game, depth) {
            Some(true) => self.right.verify(moveset, game, partial_game, depth),
            x => x
        }
    }
}
