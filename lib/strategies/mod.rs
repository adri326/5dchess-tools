use super::prelude::*;

/*? A set of non-essential strategies that you may use to optimize the tree searches and other computationally expensive tasks.
Following is a list of the submodules and what they include:

- [`legal`](./legal.rs): Contains legality-proving strategies (`LegalMoves`, etc.)
- [`misc`](./misc.rs): Contains miscellaneous strategies that do not fit in the above categories
*/

pub mod legal;
pub mod misc;
