use crate::game::*;

/** Preorder tree traversal algorithm, or "bubble down" algorithm (down as in https://www.reddit.com/r/ProgrammerHumor/comments/kk5ng1/finally_after_years_of_search_i_found_a_real_tree/).
    Traverses a game state by going forward in time.
    Branches as per the arrows, ie. whenever a new timeline emerges from the current board.

    This functions borrows boards **mutably** and gives them to a user-defined function.
    If you wish to borrow the boards immutably, use `bubble_down` instead.

    ## Example:

    ```
     /- D : +1L
     |
    A - B - C : 0L
    ^   ^   ^
    T0w T0b T1w
    ```

    When given this game state as the `game` argument and `⟨0, 0⟩` as the `coords` argument, the user-defined `function` will be called on the boards in the following order:
    - `x₁ = function(A, ⟨0, 0⟩, x₀)`
    - `x₂ = function(B, ⟨0, 1⟩, x₁)`
    - `x₃ = function(C, ⟨0, 2⟩, x₂)`
    - `x₄ = function(D, ⟨1, 1⟩, x₁)`
**/
pub fn bubble_down_mut<'a, F, T>(
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
        bubble_down_mut(game, coords, function, new_state.clone());
    }
}

/** Preorder tree traversal algorithm, or "bubble down" algorithm (down as in https://www.reddit.com/r/ProgrammerHumor/comments/kk5ng1/finally_after_years_of_search_i_found_a_real_tree/).
    Traverses a game state by going forward in time.
    Branches as per the arrows, ie. whenever a new timeline emerges from the current board.

    This functions borrows boards **immutably** and gives them to a user-defined function.
    If you wish to borrow the boards mutably, use `bubble_down_mut` instead.

    ## Example:

    ```
     /- D : +1L
     |
    A - B - C : 0L
    ^   ^   ^
    T0w T0b T1w
    ```

    When given this game state as the `game` argument and `⟨0, 0⟩` as the `coords` argument, the user-defined `function` will be called on the boards in the following order:
    - `x₁ = function(A, ⟨0, 0⟩, x₀)`
    - `x₂ = function(B, ⟨0, 1⟩, x₁)`
    - `x₃ = function(C, ⟨0, 2⟩, x₂)`
    - `x₄ = function(D, ⟨1, 1⟩, x₁)`
**/
pub fn bubble_down<'a, F, T>(game: &'a Game, coords: (Layer, Time), function: F, initial_state: T)
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
            bubble_down(
                game,
                (timeline.index, coords.1 + 1),
                function,
                new_state.clone(),
            );
        }
    }
}

/** Upward tree traversal, or "bubble up" algorithm.
    Traverses a game state by going backwards in time, unrolling the history of a given board.
    No branching occurs.

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

    When given this game state as the `game` argument and `⟨1, 1⟩` as the `coords` argument, the user-defined `function` will be called on the boards in the following order:
    - `x₁ = function(D, ⟨1, 1⟩, x₀)`
    - `x₂ = function(A, ⟨0, 0⟩, x₁)`

    Similarly, if `coords = ⟨0, 2⟩`:

    - `x₁ = function(C, ⟨0, 2⟩, x₀)`
    - `x₂ = function(B, ⟨0, 1⟩, x₁)`
    - `x₃ = function(A, ⟨0, 0⟩, x₂)`
**/
pub fn bubble_up_mut<'a, F, T>(
    game: &'a mut Game,
    mut coords: (Layer, Time),
    function: F,
    initial_state: T,
) -> T
where
    T: Clone,
    F: (Fn(&mut Board, (Layer, Time), T) -> T) + Copy,
{
    let mut state = initial_state;

    while let Some(board) = game.boards.get_mut(&coords) {
        state = function(board, coords, state);

        if let Some(starts_from) = game.info.get_timeline(coords.0).unwrap().starts_from {
            if starts_from.1 == coords.1 + 1 {
                coords = starts_from;
                continue;
            }
        }

        coords = (coords.0, coords.1 - 1);
    }

    state
}

/** Upward tree traversal, or "bubble up" algorithm.
    Traverses a game state by going backwards in time, unrolling the history of a given board.
    No branching occurs.

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

    When given this game state as the `game` argument and `⟨1, 1⟩` as the `coords` argument, the user-defined `function` will be called on the boards in the following order:
    - `x₁ = function(D, ⟨1, 1⟩, x₀)`
    - `x₂ = function(A, ⟨0, 0⟩, x₁)`

    Similarly, if `coords = ⟨0, 2⟩`:

    - `x₁ = function(C, ⟨0, 2⟩, x₀)`
    - `x₂ = function(B, ⟨0, 1⟩, x₁)`
    - `x₃ = function(A, ⟨0, 0⟩, x₂)`
**/
pub fn bubble_up<'a, F, T>(
    game: &'a Game,
    mut coords: (Layer, Time),
    function: F,
    initial_state: T,
) -> T
where
    T: Clone,
    F: (Fn(&Board, (Layer, Time), T) -> T) + Copy,
{
    let mut state = initial_state;

    while let Some(board) = game.boards.get(&coords) {
        state = function(board, coords, state);

        if let Some(starts_from) = game.info.get_timeline(coords.0).unwrap().starts_from {
            if starts_from.1 == coords.1 + 1 {
                coords = starts_from;
                continue;
            }
        }

        coords = (coords.0, coords.1 - 1);
    }

    state
}
