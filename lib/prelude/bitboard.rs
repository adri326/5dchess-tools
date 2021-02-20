use super::*;

/// Slower, but supports boards up to 11x11
#[cfg(bitboard128)]
pub type BitBoardPrimitive = u128;
/// Faster, but only supports boards up to 8x8
#[cfg(not(bitboard128))]
pub type BitBoardPrimitive = u64;

/** The number of bitboards that there are.
    If your pieces can't be expressed using the basic 5D Chess pieces, you'll have to add new bitboards and increase this amount.
    Currently, there are 11 piece movement kinds that are used:
    1. pawn capture
    2. brawn capture (minus pawn capture)
    3. 1-agonal leaper (wazir)
    4. 2-agonal leaper (ferz)
    5. 3-agonal leaper (rhino)
    6. 4-agonal leaper (wolf)
    7. 1-agonal rider (rook)
    8. 2-agonal rider (bishop)
    9. 3-agonal rider (unicorn)
    10. 4-agonal rider (dragon)
    11. ⟨2,1,0,0⟩-leaper (knight)
**/
pub const N_BITBOARDS: usize = 11;

/**
    Contains the bitboards for the different piece kinds of each player.
    They are named after their respective, basic piece movements.

    Note that bitboards go from left to right and from bottom to top; with a 3x3 bitboard:

    ```
    789
    456
    123
    ```
**/
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BitBoards {
    // White's pieces
    pub white: [BitBoardPrimitive; N_BITBOARDS],
    pub white_royal: BitBoardPrimitive,
    pub white_movable: BitBoardPrimitive,

    // Black's pieces
    pub black: [BitBoardPrimitive; N_BITBOARDS],
    pub black_royal: BitBoardPrimitive,
    pub black_movable: BitBoardPrimitive,

    pub castle: BitBoardPrimitive,
}

impl BitBoards {
    pub fn set(&mut self, mask: &BitBoardMask, shift: u32) {
        for n in 0..N_BITBOARDS {
            self.white[n] = (self.white[n] & !(1 << shift)) | (mask.white[n] as BitBoardPrimitive) << shift;
            self.black[n] = (self.black[n] & !(1 << shift)) | (mask.black[n] as BitBoardPrimitive) << shift;
        }

        self.white_royal = (self.white_royal & !(1 << shift)) | (mask.white_royal as BitBoardPrimitive) << shift;
        self.black_royal = (self.black_royal & !(1 << shift)) | (mask.black_royal as BitBoardPrimitive) << shift;

        self.white_movable = (self.white_movable & !(1 << shift)) | (mask.white_movable as BitBoardPrimitive) << shift;
        self.black_movable = (self.black_movable & !(1 << shift)) | (mask.black_movable as BitBoardPrimitive) << shift;
    }

    /// Assumes that `pieces` fits!
    pub fn from_pieces(pieces: &Vec<Tile>) -> Self {
        RSHIFT_MASK[0];
        let mut white = [0; N_BITBOARDS];
        let mut white_royal = 0;
        let mut white_movable = 0;

        let mut black = [0; N_BITBOARDS];
        let mut black_royal = 0;
        let mut black_movable = 0;

        for n in 0..(pieces.len() as u32) {
            let mask = pieces[n as usize].bitboard_mask();
            for o in 0..N_BITBOARDS {
                white[o] |= (mask.white[o] as BitBoardPrimitive) << n;
                black[o] |= (mask.black[o] as BitBoardPrimitive) << n;
            }
            white_royal |= (mask.white_royal as BitBoardPrimitive) << n;
            white_movable |= (mask.white_movable as BitBoardPrimitive) << n;
            black_royal |= (mask.black_royal as BitBoardPrimitive) << n;
            black_movable |= (mask.black_movable as BitBoardPrimitive) << n;
        }

        Self {
            white,
            white_royal,
            white_movable,

            black,
            black_royal,
            black_movable,

            castle: 0,
        }
    }

    pub fn set_castle(&mut self, castle: Option<(u32, u32)>) {
        match castle {
            Some((i1, i2)) => {
                self.castle = (1 << i1) | (1 << i2);
            }
            None => {
                self.castle = 0;
            }
        }
    }
}

impl Default for BitBoards {
    fn default() -> Self {
        Self {
            // White's pieces
            white: [0; N_BITBOARDS],
            white_royal: 0,
            white_movable: !0,

            // Black's pieces
            black: [0; N_BITBOARDS],
            black_royal: 0,

            black_movable: !0,
            castle: 0,
        }
    }
}

pub const VOID_BITBOARDS: BitBoards = BitBoards {
    white: [0; N_BITBOARDS],
    white_royal: 0,
    white_movable: 0,
    black: [0; N_BITBOARDS],
    black_royal: 0,
    black_movable: 0,
    castle: 0,
};

/// Contains the state of a piece, to then be put into a bitboard
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BitBoardMask {
    // White's pieces
    pub white: [bool; N_BITBOARDS],
    pub white_royal: bool,
    pub white_movable: bool,

    // Black's pieces
    pub black: [bool; N_BITBOARDS],
    pub black_royal: bool,
    pub black_movable: bool,
}

impl Default for BitBoardMask {
    fn default() -> Self {
        Self {
            // White's pieces
            white: [false; N_BITBOARDS],
            white_royal: false,
            white_movable: true,

            // Black's pieces
            black: [false; N_BITBOARDS],
            black_royal: false,
            black_movable: true,
        }
    }
}

/** Gets the right mask for the given board width and Δx
    You should apply this mask before bitshifting your bitboard, as to prevent pieces to wrap around the width of the board; for instance:

    ```
    x = 0b0100101000000100:
    0010
    0101
    0000
    0010
    ```

    A naive rshift gives us:

    ```
    x<<1 = 0b1001010000001000:
    1001
    0010
    0000
    0001
    ```

    Whereas, if we apply this mask:

    ```
    (x & get_bitboard_mask(4, 1)) << 1 = 0b1000010000001000:
    0001
    0010
    0000
    0001
    ```

    Assumes that 0 ≤ width < MAX_BITBOARD_WIDTH and -width ≤ Δx ≤ width.
**/
#[inline]
pub fn get_bitboard_mask(width: Physical, dx: Physical) -> BitBoardPrimitive {
    if dx > 0 {
        LSHIFT_MASK[(width as usize - 1) * MAX_BITBOARD_WIDTH + dx as usize]
    } else {
        RSHIFT_MASK[(width as usize - 1) * MAX_BITBOARD_WIDTH + (-dx) as usize]
    }
}

/**
    Shifts a bitboard by a given Δx and Δy, also applying the correct mask.

    ## Example

    ```
    x = 0b0100101000000100:
    0010
    0101
    0000
    0010
    ```

    ```
    bitboard_shift(x, -1, 0, 4, 4) = 0b0010010100000010:
    0100
    1010
    0000
    0100
    ```

    ```
    bitboard_shift(x, 1, 0, 4, 4) = 0b1000010000001000:
    0001
    0010
    0000
    0001
    ```

    ```
    bitboard_shift(x, 0, 1, 4, 4) = 0b1010000001000000:
    0101
    0000
    0010
    0000
    ```

    ```
    bitboard_shift(x, 0, -1, 4, 4) = 0b0000010010100000:
    0000
    0010
    0101
    0000
    ```
**/
#[inline]
pub fn bitboard_shift(mut bitboard: BitBoardPrimitive, dx: Physical, dy: Physical, width: Physical, height: Physical) -> BitBoardPrimitive {
    let width = width as usize;
    let height = height as usize;

    if dx > 0 {
        bitboard &= LSHIFT_MASK[(width - 1) * MAX_BITBOARD_WIDTH + dx as usize];
        bitboard <<= dx as usize;
    } else {
        bitboard &= RSHIFT_MASK[(width - 1) * MAX_BITBOARD_WIDTH + (-dx) as usize];
        bitboard >>= (-dx) as usize;
    }

    if dy > 0 {
        bitboard <<= (dy as usize) * (width)
    } else {
        bitboard >>= ((-dy) as usize) * (width)
    }

    if cfg!(bitboard128) {
        if width * height == 128 {
            bitboard
        } else {
            bitboard & ((1 << (width * height)) - 1)
        }
    } else {
        if width * height == 64 {
            bitboard
        } else {
            bitboard & ((1 << (width * height)) - 1)
        }
    }
}

#[cfg(bitboard128)]
pub const MAX_BITBOARD_WIDTH: usize = 11;
#[cfg(not(bitboard128))]
pub const MAX_BITBOARD_WIDTH: usize = 8;

lazy_static! {
    pub static ref PIECE_MASKS: [BitBoardMask; 2 * N_PIECES + 2] = {
        let mut res: [BitBoardMask; 2 * N_PIECES + 2] = [BitBoardMask::default(); 2 * N_PIECES + 2];
        res[1].white_movable = false;
        res[1].black_movable = false;

        // Number wall goes brr
        let kernel: [([u8; N_BITBOARDS], u8); N_PIECES] = [
            ([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 0),
            ([0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0], 0),
            ([0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0], 0),
            ([0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0], 1),
            ([1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0], 0),
            ([0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0], 0),
            ([0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0], 1),
        ];

        for (i, k) in kernel.iter().enumerate() {
            let mut transformed_kernel: [bool; N_BITBOARDS] = [false; N_BITBOARDS];

            for n in 0..N_BITBOARDS {
                transformed_kernel[n] = k.0[n] > 0;
            }

            // White
            res[i + N_PIECES + 2] = BitBoardMask {
                white: transformed_kernel,
                white_royal: k.1 > 0,
                white_movable: false,
                black: [false; N_BITBOARDS],
                black_royal: false,
                black_movable: k.1 > 0,
            };

            // Black
            res[i + 2] = BitBoardMask {
                white: [false; N_BITBOARDS],
                white_royal: false,
                white_movable: k.1 > 0,
                black: transformed_kernel,
                black_royal: k.1 > 0,
                black_movable: false,
            };
        }

        res
    };

    // The length needs a +1 to allow for `width = MAX_BITBOARD_WIDTH`, `shift = width`
    pub static ref RSHIFT_MASK: [BitBoardPrimitive; MAX_BITBOARD_WIDTH * MAX_BITBOARD_WIDTH + 1] = {
        let mut res = [0; MAX_BITBOARD_WIDTH * MAX_BITBOARD_WIDTH + 1];

        for width in 1..=MAX_BITBOARD_WIDTH {
            for shift in 0..width {
                let mut kernel: BitBoardPrimitive = (1 << width) - (1 << shift);
                let mut mask = kernel;
                while kernel > 0 {
                    kernel <<= width;
                    mask |= kernel;
                }
                res[(width - 1) * MAX_BITBOARD_WIDTH + shift] = mask;
            }
        }

        res
    };

    pub static ref LSHIFT_MASK: [BitBoardPrimitive; MAX_BITBOARD_WIDTH * MAX_BITBOARD_WIDTH + 1] = {
        let mut res = [0; MAX_BITBOARD_WIDTH * MAX_BITBOARD_WIDTH + 1];

        for width in 1..=MAX_BITBOARD_WIDTH {
            for shift in 0..width {
                let mut kernel: BitBoardPrimitive = (1 << (width - shift)) - 1;
                let mut mask = kernel;
                while kernel > 0 {
                    kernel <<= width;
                    mask |= kernel;
                }
                res[(width - 1) * MAX_BITBOARD_WIDTH + shift] = mask;
            }
        }

        res
    };
}
