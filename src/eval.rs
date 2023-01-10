use crate::{consts::*, position::Position};

macro_rules! count {($bb:expr) => {$bb.count_ones() as i16}}

#[inline(always)]
fn wspans(mut pwns: u64) -> u64 {
    pwns |= pwns << 8;
    pwns |= pwns << 16;
    pwns |= pwns << 32;
    pwns << 8
}

#[inline(always)]
fn bspans(mut pwns: u64) -> u64 {
    pwns |= pwns >> 8;
    pwns |= pwns >> 16;
    pwns |= pwns >> 32;
    pwns >> 8
}

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
    threat: i16,
    defend: i16,
    attack: i16,
}

fn major_mobility(pc: usize, mut attackers: u64, occ: u64, friends: u64, danger: &mut i16, ksqs: u64) -> MajorMobility {
    let mut from: usize;
    let mut attacks: u64;
    let mut ret: MajorMobility = MajorMobility::default();
    while attackers > 0 {
        from = attackers.trailing_zeros() as usize;
        attackers &= attackers - 1;
        attacks = match pc {
            KNIGHT => NATT[from],
            BISHOP => batt(from, occ),
            ROOK => ratt(from, occ),
            QUEEN => ratt(from, occ) | batt(from, occ),
            _ => unimplemented!("only implement the four major pieces"),
        };
        ret.threat += count!(attacks & (occ & !friends));
        ret.defend += count!(attacks & friends);
        ret.attack += count!(attacks & !occ);
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

    // pawn progression
    for i in 0..5 {
        pos.vals[PAWN_PROGRESSION + i] = count!(wp & PAWN_RANKS[i + 1]) - count!(bp & PAWN_RANKS[4 - i]);
    }

    // king danger stuff
    let mut wking_danger: i16 = 0;
    let mut bking_danger: i16 = 0;
    let wking_sqs: u64 = KATT[bitboards[WHITE][KING].trailing_zeros() as usize];
    let bking_sqs: u64 = KATT[bitboards[BLACK][KING].trailing_zeros() as usize];

    // set major piece mobility values
    for i in 0..MAJOR_PIECES {
        // rooks don't block each other, rooks and bishops don't block queen, queen blocks nothing
        let (tw, tb): (u64, u64) = match i + 1 {
            ROOK => (
                bitboards[WHITE][ROOK] ^ bitboards[WHITE][QUEEN] ^ bitboards[BLACK][KING],
                bitboards[BLACK][ROOK] ^ bitboards[BLACK][QUEEN] ^ bitboards[WHITE][KING],
            ),
            QUEEN => (
                bitboards[WHITE][BISHOP] ^ bitboards[WHITE][ROOK] ^ bitboards[WHITE][QUEEN] ^ bitboards[BLACK][KING],
                bitboards[BLACK][BISHOP] ^ bitboards[BLACK][ROOK] ^ bitboards[BLACK][QUEEN] ^ bitboards[WHITE][KING],
            ),
            BISHOP => (bitboards[WHITE][QUEEN] ^ bitboards[BLACK][KING] ,bitboards[BLACK][QUEEN] ^ bitboards[WHITE][KING]),
            _ => (0, 0)
        };
        let w_maj_mob: MajorMobility = major_mobility(i + 1, bitboards[WHITE][i + 1], occ ^ tw, sides[WHITE], &mut bking_danger, bking_sqs);
        let b_maj_mob: MajorMobility = major_mobility(i + 1, bitboards[BLACK][i + 1], occ ^ tb, sides[BLACK], &mut wking_danger, wking_sqs);
        pos.vals[MAJOR_THREAT + i] = w_maj_mob.threat - b_maj_mob.threat;
        pos.vals[MAJOR_DEFEND + i] = w_maj_mob.defend - b_maj_mob.defend;
        pos.vals[MAJOR_ATTACK + i] = w_maj_mob.attack - b_maj_mob.attack;
    }

    // set pawn and king danger values
    pos.vals[KING_DANGER] = wking_danger - bking_danger;
    pos.vals[PAWN_SHIELD] = count!(wp & wking_sqs) - count!(bp & bking_sqs);

    // passed pawns
    pos.vals[PAWN_PASSED] = count!(wp & !bspans(bp | bp_att)) - count!(bp & !wspans(wp | wp_att));
}
