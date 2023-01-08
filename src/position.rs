use crate::{S, NUM_PARAMS, TPHASE};

const CHARS: [char; 12] = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];

#[derive(Default)]
pub struct Position {
    pub vals: [i16; NUM_PARAMS],
    phase: i16,
    result: f32,
}

impl Position {
    pub fn from_epd(epd: &str) -> Self {
        let mut pos = Position::default();
        let (mut row, mut col): (u16, u16) = (7, 0);
        let mut bitboards: [[u64; 6]; 2] = [[0; 6]; 2];
        for ch in epd.chars() {
            if ch == '/' {row -= 1; col = 0;}
            else if ch == ' ' {break}
            else if ('1'..='8').contains(&ch) {col += ch.to_digit(10).expect("hard coded") as u16}
            else if let Some(idx) = CHARS.iter().position(|&element| element == ch) {
                let c: usize = idx / 6;
                let (pc, sq): (u16, u16) = (idx as u16 - 6 * c as u16, 8 * row + col);
                bitboards[c][pc as usize] |= 1 << sq;
                pos.phase += [0, 1, 1, 2, 4, 0, 0][pc as usize];
                col += 1;
            }
        }
        // set material vals
        for i in 0..5 {
            pos.vals[i] = bitboards[0][i].count_ones() as i16 - bitboards[1][i].count_ones() as i16;
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