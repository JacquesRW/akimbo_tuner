use crate::{consts::TPHASE, NUM_PARAMS};
use std::ops::{AddAssign, Mul, Index, IndexMut};

#[derive(Clone, Copy, Debug, Default)]
pub struct S(i16, i16);

impl AddAssign<S> for S {
    fn add_assign(&mut self, rhs: S) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Mul<S> for i8 {
    type Output = S;
    fn mul(self, rhs: S) -> Self::Output {
        S(self as i16 * rhs.0, self as i16 * rhs.1)
    }
}

impl Index<bool> for S {
    type Output = i16;
    fn index(&self, index: bool) -> &Self::Output {
        if index {&self.1} else {&self.0}}
}

impl IndexMut<bool> for S {
    fn index_mut(&mut self, index: bool) -> &mut Self::Output {
        if index {&mut self.1} else {&mut self.0}
    }
}

impl S {
    pub const ONES: Self = Self(1, 1);

    pub const INIT: [Self; NUM_PARAMS] = {
        let mut ret = [Self(0, 0); NUM_PARAMS];
        ret[0] = Self(100, 100);
        ret[1] = Self(300, 300);
        ret[2] = Self(300, 300);
        ret[3] = Self(500, 500);
        ret[4] = Self(900, 900);
        ret
    };

    #[inline]
    pub fn taper(self, phase: i16) -> i32 {
        let p = phase as i32;
        (p * self.0 as i32 + (TPHASE - p) * self.1 as i32) / TPHASE
    }

    pub fn fancy(&self) -> String {
        format!("S({: >3},{: >4})", self.0, self.1)
    }
}