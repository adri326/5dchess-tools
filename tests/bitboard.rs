use chess5dlib::prelude::*;

#[test]
fn test_shift_masks() {
    for width in 1..=MAX_BITBOARD_WIDTH {
        for shift in 0..width {
            let rshift_mask = RSHIFT_MASK[(width - 1) * MAX_BITBOARD_WIDTH + shift];
            let lshift_mask = LSHIFT_MASK[(width - 1) * MAX_BITBOARD_WIDTH + shift];

            assert_eq!(lshift_mask, get_bitboard_mask(width as Physical, shift as Physical));
            assert_eq!(rshift_mask, get_bitboard_mask(width as Physical, -(shift as Physical)));

            assert_eq!(rshift_mask & ((1 << width) - 1), (rshift_mask >> width) & ((1 << width) - 1));
            assert_eq!(rshift_mask & ((1 << width) - 1), (rshift_mask >> (2 * width)) & ((1 << width) - 1));
            assert_eq!(lshift_mask & ((1 << width) - 1), (lshift_mask >> width) & ((1 << width) - 1));
            assert_eq!(lshift_mask & ((1 << width) - 1), (lshift_mask >> (2 * width)) & ((1 << width) - 1));

            for n in 0..shift {
                assert_eq!(rshift_mask & (1 << n), 0);
                assert_eq!(lshift_mask & (1 << (width - n - 1)), 0);
            }
            for n in shift..width {
                assert_eq!(rshift_mask & (1 << n), (1 << n));
                assert_eq!(lshift_mask & (1 << (width - n - 1)), (1 << (width - n - 1)));
            }
        }
    }

    let x: BitBoardPrimitive = 0b0100101000000100;
    assert_eq!(bitboard_shift(x, 1, 0, 4, 4), 0b1000010000001000);
    assert_eq!(bitboard_shift(x, 0, 1, 4, 4), 0b1010000001000000);
    assert_eq!(bitboard_shift(x, -1, 0, 4, 4), 0b0010010100000010);
    assert_eq!(bitboard_shift(x, 0, -1, 4, 4), 0b0000010010100000);
    assert_eq!(bitboard_shift(bitboard_shift(x, 0, 1, 4, 4), 1, 0, 4, 4), bitboard_shift(x, 1, 1, 4, 4));
    assert_eq!(bitboard_shift(bitboard_shift(x, 0, 1, 4, 4), -2, 0, 4, 4), bitboard_shift(x, -2, 1, 4, 4));
    assert_eq!(bitboard_shift(bitboard_shift(x, 2, 0, 4, 4), 0, -1, 4, 4), bitboard_shift(x, 2, -1, 4, 4));
}
