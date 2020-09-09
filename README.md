# 5dchess-tools

Programming set of tools to analyze [5D Chess](https://5dchesswithmultiversetimetravel.com/) games.

## Installation

Clone this repository:

```sh
git clone https://github.com/adri326/5dchess-tools/
cd 5dchess-tools
```

Then build or run it using `cargo`:

```sh
cargo run path/to/game.json
```

The current, included executable will read a JSON file (outputted by [this parser](https://github.com/adri326/5dchess-notation/)) and proceed to run calculations on it.

## Usage

The library half of this tool is labelled as `chess5dlib` (the executable `chess5dtools`).

- The various structures making a game's state can be found in `chess5dlib::game` (`/lib/game.rs`).
- Per-board move-related logic can be found in `chess5dlib::moves` (`/lib/moves.rs`).
- Moveset-related logic can be found in `chess5dlib::moveset` (`/lib/moveset.rs`).
  Note that as I am writing this, these functions are heavily oriented towards a branch factor-limited, tree-based analysis.
- Board scoring logic can be found in `chess5dlib::resolve` (`/lib/resolve.rs`, might be renamed later)
- αβ-pruned search and other tree-based search algorithms can be found in `chess5dlib::tree`

## Notes

This game can reach very complex states (multi-dimensional series of checks, having to create several timelines in a specific order, etc.).
Soundness over checkmate proof is complicated to achieve and has been found to sometimes be extremely computationally expensive.

It could be very expensive to list out all of the possible movesets (sets of moves per turn), thus the algorithms here are based around a lazy method of generating these movesets.
This method relies on a two-pass analysis of the moves:

- moves are listed for every board
- for every board, moves are scored and illegal moves are pruned (**first pass**),i n the same way that the game shows you a red indicator when you try out these moves in the original game
- moves are lazily combined, based on their ordering made in the first pass
- movesets are scored and illegal movesets are pruned (**second pass**)
