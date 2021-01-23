# 5dchess-tools

Programming set of tools to analyze [5D Chess](https://5dchesswithmultiversetimetravel.com/) games.

***Note:** You are currently seeing the WIP, future `0.2` release. The information below is partially outdated!*

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

### As a dependency

Add the following to your `Cargo.toml`:

```
[dependencies.chess5dtools]
version = "0.1"
git = "https://github.com/adri326/5dchess-tools"
```

You can then import the different modules in your code, for instance:

```rs
use chess5dlib::game::*;
```

## Usage

The library half of this tool is labelled as `chess5dlib` (the executable and package `chess5dtools`).

You should always include the `chess5dlib::prelude` module, which contains all of the necessary structures and utilities for the other functions:

```rs
use chess5dlib::prelude::*;

fn main() {
    let pawn = Piece::new(PieceKind::Pawn, false, true);
}
```

You'll also want to include the `chess5dlib::parse` module if you wish to read files:

```rs
use chess5dlib::prelude::*;
use chess5dlib::parse::parse;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let mut file = File::open("tests/games/standard-empty.json")?;
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;

    let game = parse(&raw).expect("Couldn't parse game!");

    println!("{:?}", game.get_board((0, 0))); // Prints the starting board at ⟨0, 0⟩
}
```

### Testing

Firstly, run `git submodule update --init --recursive` to install [5dchess-notation](https://github.com/adri326/5dchess-notation); the parser from this dependency is used to convert the games from [5d-chess-db](https://gitlab.com/alexbay218/5d-chess-db) to JSON.

You'll then want to do:

```sh
cd 5dchess-notation
npm i # Installs the dependencies of 5dchess-notation
cd ..
./prepare-tests.sh # Downloads 5d-chess-db and parses games into JSON (this will take a minute)
```

Finally, you can run `cargo test` or `cargo bench`.

### Profiling

For profiling, you'll need one additional tool, [`coz`](https://github.com/plasma-umass/coz), installed on your system.
Once it's installed, run `cargo build --release --example profiling` to build the profiling binary.
You can then profile the library by running:

```sh
coz run --source-scope $(pwd)/% --- ./target/release/examples/profiling <duration (seconds)> <thread count>
```

The resulting profile will be put in a file called `profile.coz`, which you can import and analyze [here](https://plasma-umass.org/coz/).

For more information, have a look at [`coz`'s repository](https://github.com/plasma-umass/coz)!

## Notes

This game can reach very complex states (multi-dimensional series of checks, having to create several timelines in a specific order, etc.).
Soundness over checkmate proof is complicated to achieve and has been found to sometimes be extremely computationally expensive.

It could be very expensive to list out all of the possible movesets (sets of moves per turn), thus the algorithms here are based around a lazy method of generating these movesets.
This method relies on a two-pass analysis of the moves:

- moves are lazily listed for every board
- for every board, moves are scored (*not yet implemented*) and illegal moves are pruned (**first pass**), in the same way that the game shows you a red indicator when you try out these moves in the original game
- moves are lazily combined, based on their ordering made in the first pass
- movesets are scored (*not yet implemented*) and illegal movesets are pruned (**second pass**)
