pub const TPHASE: i32 = 24;
pub const SF: [i16; 2] = [1, -1];

// number of things
pub const NUM_PARAMS: usize = PAWN_PASSED + 1;
pub const NUM_VALS: usize = NUM_PARAMS;
pub const PST_SQUARES: usize = 24;
pub const DISTINCT_KNIGHT_ATTACKS: usize = 9;
pub const DISTINCT_BISHOP_ATTACKS: usize = 14;
pub const DISTINCT_ROOK_ATTACKS: usize = 15;
pub const MAJOR_MOBILITY: [usize; 3] = [MOBILITY_KNIGHT, MOBILITY_BISHOP, MOBILITY_ROOK];

// storage locations of certain features
pub const PAWN_PST: usize = 5;
pub const PAWN_SHIELD: usize = PAWN_PST + PST_SQUARES;
pub const MOBILITY_KNIGHT: usize = PAWN_SHIELD + 1;
pub const MOBILITY_BISHOP: usize = MOBILITY_KNIGHT + DISTINCT_KNIGHT_ATTACKS;
pub const MOBILITY_ROOK: usize = MOBILITY_BISHOP + DISTINCT_BISHOP_ATTACKS;
pub const KING_LINEAR: usize = MOBILITY_ROOK + DISTINCT_ROOK_ATTACKS;
pub const KING_QUADRATIC: usize = KING_LINEAR + 1;
pub const PAWN_PASSED: usize = KING_QUADRATIC + 1;

// pieces
pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;
pub const WHITE: usize = 0;
pub const BLACK: usize = 1;

// A file and ~(H file)
pub const FILE: u64 = 0x0101010101010101;
pub const NOTH: u64 = !(FILE << 7);

// diagonals
pub const DIAGS: [u64; 15] = [
    0x0100000000000000, 0x0201000000000000, 0x0402010000000000, 0x0804020100000000, 0x1008040201000000,
    0x2010080402010000, 0x4020100804020100, 0x8040201008040201, 0x0080402010080402, 0x0000804020100804,
    0x0000008040201008, 0x0000000080402010, 0x0000000000804020, 0x0000000000008040, 0x0000000000000080,
];

/// Macro for calculating tables (until const fn pointers are stable).
macro_rules! init {
    ($idx:ident, $init:expr, $($rest:tt)+) => {{
        let mut res = [$init; 64];
        let mut $idx = 0;
        while $idx < 64 {
            res[$idx] = {$($rest)+};
            $idx += 1;
        }
        res
    }};
}

// knight attacks
pub const NATT: [u64; 64] = init!(idx, 0, {
    let n = 1 << idx;
    let h1 = ((n >> 1) & 0x7f7f7f7f7f7f7f7f) | ((n << 1) & 0xfefefefefefefefe);
    let h2 = ((n >> 2) & 0x3f3f3f3f3f3f3f3f) | ((n << 2) & 0xfcfcfcfcfcfcfcfc);
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
});

// king attacks
pub const KATT: [u64; 64] = init!(idx, 0, {
    let mut k = 1 << idx;
    k |= (k << 8) | (k >> 8);
    k |= ((k & !FILE) >> 1) | ((k & NOTH) << 1);
    k ^ (1 << idx)
});

// bishop masks
pub const BMASKS: [Mask; 64] = init!(idx, Mask { bit: 0, right: 0, left: 0, file: 0 },
    let bit = 1 << idx;
    Mask { bit, right: bit ^ DIAGS[(7 + (idx & 7) - (idx >> 3))], left: bit ^ DIAGS[((idx & 7) + (idx >> 3))].swap_bytes(), file: bit.swap_bytes() }
);

// rook masks
pub const RMASKS: [Mask; 64] = init!(idx, Mask { bit: 0, right: 0, left: 0, file: 0 },
    let bit = 1 << idx;
    let left = (bit - 1) & (0xFF << (idx & 56));
    Mask { bit, right: bit ^ left ^ (0xFF << (idx & 56)), left, file: bit ^ FILE << (idx & 7) }
);

// rook attacks on rank
pub const WEST: [u64; 64] = init!(idx, 0, ((1 << idx) - 1) & (0xFF << (idx & 56)));


#[derive(Clone, Copy)]
pub struct Mask {
    pub bit: u64,
    pub right: u64,
    pub left: u64,
    pub file: u64,
}

pub const PST_IDX: [u8; 64] = init!(idx, 0, (((idx / 8) * 4).saturating_sub((4 - ((idx & 7) > 3) as i16 - (idx & 7) as i16).unsigned_abs() as usize)) as u8);
