use crate::{S, NUM_PARAMS, consts::*, eval::major_mobility};

const CHARS: [char; 12] = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];

#[derive(Default, Debug)]
pub struct Position {
    vals: [i16; NUM_PARAMS],
    phase: i16,
    result: f32,
}

impl Position {
    pub fn from_epd(epd: &str) -> Self {
        let mut pos = Position::default();
        let (mut row, mut col): (u16, u16) = (7, 0);
        let mut bitboards: [[u64; 6]; 2] = [[0; 6]; 2];
        let mut sides: [u64; 2] = [0; 2];
        for ch in epd.chars() {
            if ch == '/' {row -= 1; col = 0;}
            else if ch == ' ' {break}
            else if ('1'..='8').contains(&ch) {col += ch.to_digit(10).expect("hard coded") as u16}
            else if let Some(idx) = CHARS.iter().position(|&element| element == ch) {
                let c: usize = idx / 6;
                let (pc, sq): (u16, u16) = (idx as u16 - 6 * c as u16, 8 * row + col);
                bitboards[c][pc as usize] |= 1 << sq;
                sides[c] |= 1 << sq;
                pos.phase += [0, 1, 1, 2, 4, 0, 0][pc as usize];
                col += 1;
            }
        }

        // set material vals
        for i in PAWN..=QUEEN {
            pos.vals[i] = bitboards[WHITE][i].count_ones() as i16 - bitboards[BLACK][i].count_ones() as i16;
        }

        // set major piece mobility values
        let occ: u64 = sides[WHITE] | sides[BLACK];
        let wp: u64 = bitboards[WHITE][PAWN];
        let bp: u64 = bitboards[BLACK][PAWN];
        let wp_att: u64 = ((wp & !FILE) << 7) | ((wp & NOTH) << 9);
        let bp_att: u64 = ((bp & !FILE) >> 9) | ((bp & NOTH) >> 7);
        for i in 0..4 {
            let idx: usize = KING + i;
            let (w_thr, w_sup, w_oth): (i16, i16, i16) = major_mobility(i + 1, bitboards[WHITE][i + 1], occ, sides[WHITE], !bp_att);
            let (b_thr, b_sup, b_oth): (i16, i16, i16) = major_mobility(i + 1, bitboards[BLACK][i + 1], occ, sides[BLACK], !wp_att);
            pos.vals[idx] = w_thr - b_thr;
            pos.vals[idx + 4] = w_sup - b_sup;
            pos.vals[idx + 8] = w_oth - b_oth;
        }

        pos.phase = std::cmp::min(pos.phase, TPHASE as i16);
        pos.result = match &epd[(epd.len() - 6)..] {"\"1-0\";" => 1.0, "\"0-1\";" => 0.0, _ => 0.5};
        pos
    }

    #[inline]
    pub fn err(&self, k: f32, params: &[S; NUM_PARAMS]) -> f32 {
        let mut score = S::default();
        self.vals.iter().enumerate().for_each(|(i, &val)| score += val * params[i]);
        let eval = score.taper(self.phase) as f32;
        (self.result - 1.0 / (1.0 + 10f32.powf(-k * eval / 100.0))).powi(2)
    }
}