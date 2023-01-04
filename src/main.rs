use std::{
    cmp,
    fs::File,
    io::{stdin, BufRead, BufReader},
    ops::{AddAssign, Index, IndexMut, SubAssign},
    thread::{available_parallelism, scope, ScopedJoinHandle},
    time::Instant,
};

const CHARS: [char; 12] = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];
const TPHASE: i32 = 24;
const NUM_PARAMS: usize = 384;
const K: f32 = 0.4;
const STEP: f32 = 0.001;
const INIT: [[S; 64]; 6] = [[S(100, 100); 64], [S(300, 300); 64], [S(300, 300); 64], [S(500, 500); 64], [S(900, 900); 64], [S(0, 0); 64]];

#[derive(Clone, Copy, Debug)]
struct S(i16, i16);

#[derive(Default)]
struct Position {
    psts: [[u16; 16]; 2],
    counters: [u16; 2],
    phase: i16,
    result: f32,
}

struct Data {
    params: [S; NUM_PARAMS],
    positions: Vec<Position>,
    num: f32,
    size: usize,
}

impl AddAssign<S> for S {
    fn add_assign(&mut self, rhs: S) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl SubAssign<S> for S {
    fn sub_assign(&mut self, rhs: S) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
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

impl Position {
    fn from_epd(epd: &str) -> Self {
        let mut pos = Position::default();
        let (mut row, mut col): (u16, u16) = (7, 0);
        for ch in epd.chars() {
            if ch == '/' {
                row -= 1;
                col = 0;
            } else if ('1'..='8').contains(&ch) {
                col += ch.to_digit(10).expect("hard coded") as u16;
            } else if let Some(idx) = CHARS.iter().position(|&element| element == ch) {
                let c: usize = idx / 6;
                let (pc, sq): (u16, u16) = (idx as u16 - 6 * c as u16, 8 * row + col);
                pos.psts[c][pos.counters[c] as usize] = pc * 64 + (sq ^ (56 * (c as u16 ^ 1)));
                pos.counters[c] += 1;
                pos.phase += [0, 1, 1, 2, 4, 0, 0][pc as usize];
                col += 1;
            } else if ch == ' ' {
                break;
            }
        }
        pos.phase = cmp::min(pos.phase, TPHASE as i16);
        pos.result = match &epd[(epd.len() - 6)..] {
            "\"1-0\";" => 1.0,
            "\"0-1\";" => 0.0,
            _ => 0.5,
        };
        pos
    }

    #[inline]
    fn err(&self, k: f32, params: &[S; NUM_PARAMS]) -> f32 {
        let p: i32 = self.phase as i32;
        let mut score = S(0, 0);
        (0..usize::from(self.counters[0])).for_each(|i| score += params[self.psts[0][i] as usize]);
        (0..usize::from(self.counters[1])).for_each(|i| score -= params[self.psts[1][i] as usize]);
        let eval = ((p * score.0 as i32 + (TPHASE - p) * score.1 as i32) / TPHASE) as f32;
        (self.result - 1.0 / (1.0 + 10f32.powf(-k * eval / 100.0))).powi(2)
    }
}

fn error(k: f32, data: &Data) -> f32 {
    let total: f32 = scope(|s| {
        data.positions
            .chunks(data.size)
            .map(|chunk| s.spawn(|| chunk.iter().map(|pos| pos.err(k, &data.params)).sum()))
            .collect::<Vec<ScopedJoinHandle<f32>>>()
            .into_iter()
            .map(|p| p.join().unwrap_or_default())
            .sum()
    });
    total / data.num
}

fn dur(time: &Instant) -> f32 {
    time.elapsed().as_secs_f32()
}

fn main() {
    // LOADING POSITIONS
    let mut data = Data {
        params: INIT.concat().try_into().expect("hard coded"),
        positions: Vec::with_capacity(1_450_404),
        num: 0.0,
        size: 0,
    };
    let mut time: Instant = Instant::now();
    let file: File = File::open("set.epd").expect("should have provided correct file");
    BufReader::new(file).lines().into_iter().for_each(|ln| {
        data.positions.push(Position::from_epd(&ln.unwrap()));
        data.num += 1.0;
    });
    data.size = data.positions.len() / available_parallelism().expect("available").get();
    println!("positions {:.0} ({}/sec)", data.num, data.num / dur(&time));

    // OPTIMISING K VALUE
    time = Instant::now();
    let step: f32 = (error(K - STEP, &data) - error(K + STEP, &data)).signum() * STEP;
    let mut k: f32 = K;
    let (mut best, mut new): (f32, f32) = (error(k, &data), error(k + step, &data));
    while new <= best {
        k += step;
        best = new;
        new = error(k + step, &data);
    }
    println!("time {:.3}s error {best:.6} optimal k = {k:.3}", dur(&time));

    // TEXEL TUNING
    let mut cache: [S; NUM_PARAMS] = [S(1, 1); NUM_PARAMS];
    let mut improved: bool = true;
    while improved {
        time = Instant::now();
        improved = false;
        for (i, dir) in cache.iter_mut().enumerate() {
            for j in [false, true] {
                data.params[i][j] += dir[j];
                new = error(k, &data);
                if new < best {
                    best = new;
                    improved = true;
                } else {
                    data.params[i][j] -= 2 * dir[j];
                    new = error(k, &data);
                    if new < best {
                        best = new;
                        improved = true;
                        dir[j] = -dir[j];
                    } else {
                        data.params[i][j] += dir[j];
                    }
                }
            }
        }
        println!("time {:.3}s error {best:.6}", dur(&time));
    }
    (0..12).for_each(|i| println!("{:?},", &data.params[i * 64..(i + 1) * 64]));

    // WAIT FOR EXIT
    stdin().read_line(&mut String::new()).expect("parsable");
}
