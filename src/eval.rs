use crate::consts::*;

macro_rules! pop_lsb {($idx:expr, $x:expr) => {$idx = $x.trailing_zeros() as u8; $x &= $x - 1}}

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
pub struct MajorMobility {
    pub threats: i16,
    pub supports: i16,
    pub controls: i16,
}

pub fn major_mobility(pc: usize, mut attackers: u64, occ: u64, friends: u64, unprotected: u64, danger: &mut i16, ksqs: u64) -> MajorMobility {
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
        ret.threats += (attacks & (occ & !friends)).count_ones() as i16; // threats
        ret.supports += (attacks & friends).count_ones() as i16; // supports
        ret.controls += (attacks & (!occ & unprotected)).count_ones() as i16; // other safe mobility
        *danger += (attacks & ksqs).count_ones() as i16;
    }
    ret
}