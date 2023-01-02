use std::{cmp, fs::File, io::{self, BufRead}, process, time::Instant, thread};

const PIECE_CHARS: [char; 12] = ['P','N','B','R','Q','K','p','n','b','r','q','k'];
const PHASE_VALS: [i16; 7] = [0, 1, 1, 2, 4, 0, 0];
const TPHASE: i32 = 24;
const INIT: f32 = 0.4;
const STEP: f32 = 0.001;

struct Position {
    psts: [[usize; 16]; 2],
    counters: [usize; 2],
    phase: i16,
    result: f32,
}

struct Stuff {
    params: [i16; 768],
    positions: Vec<Position>,
    num: f32,
}

impl Position {
    fn from_epd(s: &str) -> Self {
        let commands: Vec<&str> = s.split("c9").map(|v| v.trim()).collect();
        let fen: Vec<char> = commands[0].split_whitespace().collect::<Vec<&str>>()[0].chars().collect();
        let mut pos = Position {psts: [[0; 16]; 2], counters: [0; 2], phase: 0, result: 0.0};
        let (mut row, mut col): (i16, i16) = (7, 0);
        for ch in fen {
            if ch == '/' { row -= 1; col = 0; }
            else if ('1'..='8').contains(&ch) { col += ch.to_string().parse::<i16>().unwrap_or(0) }
            else {
                let idx: usize = PIECE_CHARS.iter().position(|&element| element == ch).unwrap_or(6);
                let c: usize = (idx > 5) as usize;
                let pc: usize = idx - 6 * c;
                let sq: usize = (8 * row + col) as usize;
                pos.psts[c][pos.counters[c]] = pc * 64 + (sq ^ (56 * (c ^ 1)));
                pos.counters[c] += 1;
                pos.phase += PHASE_VALS[pc];
                col += 1
            }
        }
        pos.phase = cmp::min(pos.phase, TPHASE as i16);
        pos.result = match commands[1] {"\"1-0\";" => 1.0, "\"0-1\";" => 0.0, _ => 0.5};
        pos
    }

    #[inline]
    fn eval(&self, params: &[i16; 768]) -> f32 {
        let p: i32 = self.phase as i32;
        let mut mg: i16 = 0;
        let mut eg: i16 = 0;
        for i in 0..self.counters[0] {
            mg += params[self.psts[0][i]];
            eg += params[self.psts[0][i] + 384];
        }
        for i in 0..self.counters[1] {
            mg -= params[self.psts[1][i]];
            eg -= params[self.psts[1][i] + 384];
        }
        ((p * mg as i32 + (TPHASE - p) * eg as i32) / TPHASE) as f32
    }
}

impl Stuff {
    fn error_of_slice(&self, k: f32, i: usize, ppt: usize) -> f32 {
        self.positions[i * ppt .. (i + 1) * ppt].iter().fold(0.0, |err, p|
            err + (p.result - 1.0 / (1.0 + 10f32.powf(-k * p.eval(&self.params) / 100.0))).powi(2)
        )
    }
}

fn error(k: f32, stuff: &Stuff, num_threads: usize) -> f32 {
    let ppt: usize = stuff.positions.len() / num_threads;
    let total_error: f32 = thread::scope(|s|
        (0..num_threads)
            .map(|i| s.spawn(move || stuff.error_of_slice(k, i, ppt)))
            .collect::<Vec<thread::ScopedJoinHandle<f32>>>()
            .into_iter()
            .fold(0.0, |err, p| err + p.join().unwrap())
    );
    total_error / stuff.num
}

macro_rules! err {($s:expr) => {|_| {println!($s); process::exit(1);}}}
macro_rules! elapsed {($time:expr) => {$time.elapsed().as_millis() as f32 / 1000.0}}

fn main() {
    let threads = thread::available_parallelism()
        .unwrap_or_else(err!("error checking threads")).get();
    println!("{threads} threads available");
    // LOADING POSITIONS
    let mut stuff: Stuff = Stuff {
        params: [[[100; 64], [300; 64], [300; 64], [500; 64], [900; 64], [0; 64]]; 2]
            .concat().concat().try_into().unwrap(),
        positions: Vec::new(),
        num: 0.0,
    };
    let mut time: Instant = Instant::now();
    let mut n: usize = 0;
    let file: File = File::open("set.epd").unwrap_or_else(err!("error loading file"));
    for line in io::BufReader::new(file).lines() {
        let pos: Position = Position::from_epd(&line
            .unwrap_or_else(err!("error reading line")));
        n += 1;
        stuff.positions.push(pos);
    }
    stuff.num = n as f32;
    let elapsed: f32 = elapsed!(time);
    println!("loaded {n} positions in {elapsed} seconds ({}/sec)", stuff.num / elapsed);

    // OPTIMISING K VALUE
    time = Instant::now();
    let mut k: f32 = INIT;
    let mut best_error: f32 = error(k, &stuff, threads);
    let mut new_error: f32;
    let step: f32 = (error(k - STEP, &stuff, threads) - error(k + STEP, &stuff, threads)).signum() * STEP;
    loop {
        new_error = error(k + step, &stuff, threads);
        if new_error > best_error {break}
        k += step;
        best_error = new_error;
    }
    println!("optimal k: {k:.3}, error: {best_error:.6}, time: {:.2}s", elapsed!(time));

    // TEXEL TUNING
    let mut improves: [i16; 768] = [1; 768];
    let mut improved: bool = true;
    let mut count: i32 = 1;
    while improved {
        time = Instant::now();
        improved = false;
        for (i, dir) in improves.iter_mut().enumerate() {
            stuff.params[i] += *dir;
            new_error = error(k, &stuff, threads);
            if new_error < best_error {
                best_error = new_error;
                improved = true;
            } else {
                stuff.params[i] -= 2 * (*dir);
                new_error = error(k, &stuff, threads);
                if new_error < best_error {
                    best_error = new_error;
                    improved = true;
                    *dir = -(*dir);
                } else {
                    stuff.params[i] += *dir;
                }
            }
        }
        println!("epoch {count}: {:.2}s, error: {best_error:.6}", elapsed!(time));
        count += 1;
    }
    println!("Finished optimisation:");
    for i in 0..12 {println!("{:?},", &stuff.params[i * 64 .. (i + 1) * 64])}

    // WAIT FOR EXIT
    loop {
        let mut input: String = String::new();
        io::stdin().read_line(&mut input).unwrap_or_else(err!("error parsing input"));
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        if commands[0] == "quit" {break}
    }
}
