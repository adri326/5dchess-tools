use crate::game::*;

/** Preorder tree traversal algorithm, or "bubble up" algorithm.
    Traverses the game state by going forward in time.
    Branches as per the arrows, ie. whenever a new timeline emerges from the current board.

    This functions borrows boards **mutably** and gives them to a user-defined function.
    If you wish to borrow the boards immutably, use `bubble_up` instead.

    ## Example:

    ```
     /- D : +1L
     |
    A - B - C : 0L
    ^   ^   ^
    T0w T0b T1w
    ```

    When given this game state as the `game` argument and `(0, 0)` as the `coords` argument, the user-defined `function` will be called on the boards in the following order:
    - `x₁ = function(A, ⟨0, 0⟩, x₀)`
    - `x₂ = function(B, ⟨0, 1⟩, x₁)`
    - `x₃ = function(C, ⟨0, 2⟩, x₂)`
    - `x₄ = function(D, ⟨1, 1⟩, x₁)`
**/
pub fn bubble_up_mut<'a, F, T>(
    game: &'a mut Game,
    coords: (Layer, Time),
    function: F,
    initial_state: T,
) where
    T: Clone,
    F: (Fn(&mut Board, (Layer, Time), T) -> T) + Copy,
{
    let new_state = match game.boards.get_mut(&coords) {
        Some(board) => function(board, coords, initial_state),
        None => return,
    };

    let mut next_coords: Vec<(Layer, Time)> = vec![(coords.0, coords.1 + 1)];

    for timeline in game
        .info
        .timelines_white
        .iter()
        .chain(game.info.timelines_black.iter())
    {
        if timeline.starts_from == Some(coords) {
            next_coords.push((timeline.index, coords.1 + 1));
        }
    }

    for coords in next_coords.into_iter() {
        bubble_up_mut(game, coords, function, new_state.clone());
    }
}

/** Preorder tree traversal algorithm, or "bubble up" algorithm.
    Traverses the game state by going forward in time.
    Branches as per the arrows, ie. whenever a new timeline emerges from the current board.

    This functions borrows boards **immutably** and gives them to a user-defined function.
    If you wish to borrow the boards mutably, use `bubble_up_mut` instead.

    ## Example:

    ```
     /- D : +1L
     |
    A - B - C : 0L
    ^   ^   ^
    T0w T0b T1w
    ```

    When given this game state as the `game` argument and `(0, 0)` as the `coords` argument, the user-defined `function` will be called on the boards in the following order:
    - `x₁ = function(A, ⟨0, 0⟩, x₀)`
    - `x₂ = function(B, ⟨0, 1⟩, x₁)`
    - `x₃ = function(C, ⟨0, 2⟩, x₂)`
    - `x₄ = function(D, ⟨1, 1⟩, x₁)`
**/
pub fn bubble_up<'a, F, T>(game: &'a Game, coords: (Layer, Time), function: F, initial_state: T)
where
    T: Clone,
    F: (Fn(&Board, (Layer, Time), T) -> T) + Copy,
{
    let new_state = match game.boards.get(&coords) {
        Some(board) => function(board, coords, initial_state),
        None => return,
    };

    bubble_up(game, (coords.0, coords.1 + 1), function, new_state.clone());

    for timeline in game
        .info
        .timelines_white
        .iter()
        .chain(game.info.timelines_black.iter())
    {
        if timeline.starts_from == Some(coords) {
            bubble_up(
                game,
                (timeline.index, coords.1 + 1),
                function,
                new_state.clone(),
            );
        }
    }
}
