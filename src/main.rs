use std::{
    cmp,
    fs::File,
    io::{stdin, BufRead, BufReader},
    thread::{available_parallelism, scope, ScopedJoinHandle},
    time::Instant,
};

const CHARS: [char; 12] = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];
const TPHASE: i32 = 24;
const NUM_PARAMS: usize = 768;
const K: f32 = 0.4;
const STEP: f32 = 0.001;
const INIT: [[i16; 64]; 6] = [
    [100; 64], [300; 64], [300; 64], [500; 64], [900; 64], [0; 64],
];

struct Position {
    psts: [[u16; 16]; 2],
    counters: [u16; 2],
    phase: i16,
    result: f32,
}

struct Data {
    params: [i16; 768],
    positions: Vec<Position>,
    num: f32,
    chunk_size: usize,
}

impl Position {
    fn from_epd(epd: &str) -> Self {
        let mut pos = Position {
            psts: [[0; 16]; 2],
            counters: [0; 2],
            phase: 0,
            result: 0.0,
        };
        let (mut row, mut col): (u16, u16) = (7, 0);
        let mut divide: usize = 0;
        for (i, ch) in epd.chars().enumerate() {
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
                divide = i;
                break;
            }
        }
        pos.phase = cmp::min(pos.phase, TPHASE as i16);
        pos.result = match epd[divide..].split_whitespace().last() {
            Some("\"1-0\";") => 1.0,
            Some("\"0-1\";") => 0.0,
            _ => 0.5,
        };
        pos
    }

    #[inline]
    fn err(&self, k: f32, params: &[i16; NUM_PARAMS]) -> f32 {
        let p: i32 = self.phase as i32;
        let mut mg: i16 = 0;
        let mut eg: i16 = 0;
        for i in 0..self.counters[0] as usize {
            mg += params[self.psts[0][i] as usize];
            eg += params[self.psts[0][i] as usize + 384];
        }
        for i in 0..self.counters[1] as usize {
            mg -= params[self.psts[1][i] as usize];
            eg -= params[self.psts[1][i] as usize + 384];
        }
        let eval = ((p * mg as i32 + (TPHASE - p) * eg as i32) / TPHASE) as f32;
        (self.result - 1.0 / (1.0 + 10f32.powf(-k * eval / 100.0))).powi(2)
    }
}

fn error(k: f32, data: &Data) -> f32 {
    let params = &data.params;
    let total: f32 = scope(|s| {
        data.positions
            .chunks(data.chunk_size)
            .map(|chunk| s.spawn(|| chunk.iter().fold(0.0, |err, p| err + p.err(k, params))))
            .collect::<Vec<ScopedJoinHandle<f32>>>()
            .into_iter()
            .fold(0.0, |err, p| err + p.join().unwrap_or(0.0))
    });
    total / data.num
}

fn dur(time: &Instant) -> f32 {
    time.elapsed().as_millis() as f32 / 1000.0
}

fn main() {
    // LOADING POSITIONS
    let mut data = Data {
        params: [INIT; 2].concat().concat().try_into().expect("hard coded"),
        positions: Vec::new(),
        num: 0.0,
        chunk_size: 0,
    };
    let mut time: Instant = Instant::now();
    let file: File = File::open("set.epd").expect("should have provided correct file");
    data.num = BufReader::new(file).lines().into_iter().fold(0, |err, ln| {
        data.positions.push(Position::from_epd(&ln.unwrap()));
        err + 1
    }) as f32;
    data.chunk_size = data.positions.len() / available_parallelism().expect("available").get();
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
    println!("time {}s error {best:.6} optimal k = {k:.3}", dur(&time));

    // TEXEL TUNING
    let mut cache: [i16; NUM_PARAMS] = [1; NUM_PARAMS];
    let mut improved: bool = true;
    while improved {
        time = Instant::now();
        improved = false;
        for (i, dir) in cache.iter_mut().enumerate() {
            data.params[i] += *dir;
            new = error(k, &data);
            if new < best {
                best = new;
                improved = true;
            } else {
                data.params[i] -= 2 * (*dir);
                new = error(k, &data);
                if new < best {
                    best = new;
                    improved = true;
                    *dir = -(*dir);
                } else {
                    data.params[i] += *dir;
                }
            }
        }
        println!("time {}s error {best:.6}", dur(&time));
    }
    (0..12).for_each(|i| println!("{:?},", &data.params[i * 64..(i + 1) * 64]));

    // WAIT FOR EXIT
    stdin().read_line(&mut String::new()).expect("parsable");
}
