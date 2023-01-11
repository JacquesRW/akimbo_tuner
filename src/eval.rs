use crate::{consts::*, position::Position};

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
    defend: i16,
    attack: i16,
}

fn major_mobility(pc: usize, mut attackers: u64, occ: u64, friends: u64) -> MajorMobility {
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
        ret.defend += count!(attacks & friends);
        ret.attack += count!(attacks & !friends);
    }
    ret
}

pub fn set_pos_vals(pos: &mut Position, bitboards: [[u64; 6]; 2], sides: [u64; 2]) {
    // set material vals
    for i in PAWN..=QUEEN {
        pos.vals[i] = count!(bitboards[WHITE][i])- count!(bitboards[BLACK][i]);
    }

    // pawn stuff
    let occ: u64 = sides[WHITE] | sides[BLACK];
    let wp: u64 = bitboards[WHITE][PAWN];
    let bp: u64 = bitboards[BLACK][PAWN];

    // pawn progression
    for i in 0..5 {
        pos.vals[PAWN_PROGRESSION + i] = count!(wp & PAWN_RANKS[i + 1]) - count!(bp & PAWN_RANKS[4 - i]);
    }

    // king position
    let wking_idx: usize = bitboards[WHITE][KING].trailing_zeros() as usize;
    let bking_idx: usize = bitboards[BLACK][KING].trailing_zeros() as usize;
    pos.vals[KING_RANKS + wking_idx / 8] = 1;
    pos.vals[KING_RANKS + 7 - bking_idx / 8] = -1;

    // set major piece mobility values
    let q: u64 = bitboards[WHITE][QUEEN] | bitboards[BLACK][QUEEN];
    for i in 0..MAJOR_PIECES { // all ignore enemy king
        let (tw, tb): (u64, u64) = match i + 1 {
            ROOK => ( // friendly rooks compliment each other, enemy queens will move away so dont block
                bitboards[WHITE][ROOK] ^ bitboards[BLACK][KING] ^ q,
                bitboards[BLACK][ROOK] ^ bitboards[WHITE][KING] ^ q,
            ),
            QUEEN => ( // ignore friendly rooks and bishops
                bitboards[WHITE][BISHOP] ^ bitboards[WHITE][ROOK] ^ bitboards[BLACK][KING],
                bitboards[BLACK][BISHOP] ^ bitboards[BLACK][ROOK] ^ bitboards[WHITE][KING],
            ),
            BISHOP => ( // enemy rooks and queens will move out of the way so dont block
                bitboards[BLACK][KING] ^ bitboards[BLACK][ROOK] ^ q,
                bitboards[WHITE][KING] ^ bitboards[WHITE][ROOK] ^ q,
            ),
            _ => (0, 0)
        };
        let w_maj_mob: MajorMobility = major_mobility(i + 1, bitboards[WHITE][i + 1], occ ^ tw, sides[WHITE]);
        let b_maj_mob: MajorMobility = major_mobility(i + 1, bitboards[BLACK][i + 1], occ ^ tb, sides[BLACK]);
        pos.vals[MAJOR_DEFEND + i] = w_maj_mob.defend - b_maj_mob.defend;
        pos.vals[MAJOR_ATTACK + i] = w_maj_mob.attack - b_maj_mob.attack;
    }

    // set king safety values
    pos.vals[PAWN_SHIELD] = count!(wp & KATT[wking_idx]) - count!(bp & KATT[bking_idx]);
}
