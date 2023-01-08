use crate::{consts::*, position::Position};

pub const NUM_PARAMS: usize = 27;

macro_rules! pop_lsb {($idx:expr, $x:expr) => {$idx = $x.trailing_zeros() as u8; $x &= $x - 1}}
macro_rules! count {($bb:expr) => {$bb.count_ones() as i16}}

fn batt(idx: usize, occ: u64) -> u64 {
    let m: Mask = BMASKS[idx];
    let mut f: u64 = occ & m.right;
    let mut r: u64 = f.swap_bytes();
    f -= m.bit;
    r -= m.file;
    f ^= r.swap_bytes();
    f &= m.right;
    let mut f2: u64 = occ & m.left;
    r = f2.swap_bytes();
    f2 -= m.bit;
    r -= m.file;
    f2 ^= r.swap_bytes();
    f2 &= m.left;
    f | f2
}

fn ratt(idx: usize, occ: u64) -> u64 {
    let m: Mask = RMASKS[idx];
    let mut f: u64 = occ & m.file;
    let mut r: u64 = f.swap_bytes();
    f -= m.bit;
    r -= m.bit.swap_bytes();
    f ^= r.swap_bytes();
    f &= m.file;
    let mut e: u64 = m.right & occ;
    r = e & e.wrapping_neg();
    e = (r ^ (r - m.bit)) & m.right;
    let w: u64 = m.left ^ WEST[(((m.left & occ)| 1).leading_zeros() ^ 63) as usize];
    f | e | w
}

#[derive(Default)]
struct MajorMobility {
    threats: i16,
    supports: i16,
    controls: i16,
}

fn major_mobility(pc: usize, mut attackers: u64, occ: u64, friends: u64, unprotected: u64, danger: &mut i16, ksqs: u64) -> MajorMobility {
    let mut from: u8;
    let mut attacks: u64;
    let mut ret: MajorMobility = MajorMobility::default();
    attackers &= friends;
    while attackers > 0 {
        pop_lsb!(from, attackers);
        attacks = match pc {
            KNIGHT => NATT[from as usize],
            ROOK => ratt(from as usize, occ),
            BISHOP => batt(from as usize, occ),
            QUEEN => ratt(from as usize, occ) | batt(from as usize, occ),
            _ => unimplemented!("only implement the four major pieces"),
        };
        ret.threats += count!(attacks & (occ & !friends)); // threats
        ret.supports += count!(attacks & friends); // supports
        ret.controls += count!(attacks & (!occ & unprotected)); // other safe mobility
        *danger += count!(attacks & ksqs);
    }
    ret
}

pub fn set_pos_vals(pos: &mut Position, bitboards: [[u64; 6]; 2], sides: [u64; 2]) {
    // set material vals
    for i in PAWN..=QUEEN {
        pos.vals[i] = bitboards[WHITE][i].count_ones() as i16 - bitboards[BLACK][i].count_ones() as i16;
    }

    // pawn stuff
    let occ: u64 = sides[WHITE] | sides[BLACK];
    let wp: u64 = bitboards[WHITE][PAWN];
    let bp: u64 = bitboards[BLACK][PAWN];
    let wp_att: u64 = ((wp & !FILE) << 7) | ((wp & NOTH) << 9);
    let bp_att: u64 = ((bp & !FILE) >> 9) | ((bp & NOTH) >> 7);

    // king danger stuff
    let mut wking_danger: i16 = 0;
    let mut bking_danger: i16 = 0;
    let wking_sqs: u64 = KATT[bitboards[WHITE][KING].trailing_zeros() as usize];
    let bking_sqs: u64 = KATT[bitboards[BLACK][KING].trailing_zeros() as usize];

    // set major piece mobility values
    for i in 0..4 {
        let idx: usize = KING + i;
        let w_maj_mob: MajorMobility = major_mobility(i + 1, bitboards[WHITE][i + 1], occ, sides[WHITE], !bp_att, &mut bking_danger, bking_sqs);
        let b_maj_mob: MajorMobility = major_mobility(i + 1, bitboards[BLACK][i + 1], occ, sides[BLACK], !wp_att, &mut wking_danger, wking_sqs);
        pos.vals[idx] = w_maj_mob.threats - b_maj_mob.threats;
        pos.vals[idx + 4] = w_maj_mob.supports - b_maj_mob.supports;
        pos.vals[idx + 8] = w_maj_mob.controls - b_maj_mob.controls;
    }

    // set pawn and king danger values
    pos.vals[KING_SAFETY] = wking_danger - bking_danger;
    pos.vals[PAWN_SUPPORTS] = count!(sides[WHITE] & wp_att) - count!(sides[BLACK] & bp_att);
    pos.vals[PAWN_THREATS] = count!(sides[BLACK] & wp_att) - count!(sides[WHITE] & bp_att);
    pos.vals[PAWN_SHIELD] = count!(wp & wking_sqs) - count!(bp & bking_sqs);

    // pawn progression
    for i in 0..6 {
        pos.vals[21 + i] = count!(wp & PAWN_RANKS[i]) - count!(bp & PAWN_RANKS[5 - i]);
    }
}