use std::{fs::File, io::{BufRead, BufReader}, time::{Duration, Instant}, thread::{scope, Scope, ScopedJoinHandle}};

const THREADS: usize = 4;
const PHASE_VALS: [i16; 7] = [0, 1, 1, 2, 4, 0, 0];
const TPHASE: i32 = 24;
const INIT_K: f32 = 0.4;
const STEP_K: f32 = 0.001;

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
        // parsing fen
        let mut phase: i16 = 0;
        let mut psts: [[usize; 16]; 2] = [[0; 16]; 2];
        let mut counters: [usize; 2] = [0, 0];
        let mut idx: usize = 63;
        let fen: &str = commands[0].split_whitespace().collect::<Vec<&str>>()[0];
        let rows: Vec<&str> = fen.split('/').collect();
        for row in rows {
            for ch in row.chars().rev() {
                if ch == '/' { continue }
                if !ch.is_numeric() {
                    let idx2: usize = ['P','N','B','R','Q','K','p','n','b','r','q','k'].iter().position(|&element| element == ch).unwrap_or(6);
                    let (col, pc): (usize, usize) = ((idx2 > 5) as usize, idx2 - 6 * ((idx2 > 5) as usize));
                    psts[col][counters[col]] = pc * 64 + (idx ^ (56 * (col == 0) as usize));
                    counters[col] += 1;
                    phase += PHASE_VALS[pc];
                    idx -= (idx > 0) as usize;
                } else {
                    let len: usize = ch.to_string().parse::<usize>().unwrap_or(8);
                    idx -= (idx >= len) as usize * len;
                }
            }
        }
        phase = std::cmp::min(phase, TPHASE as i16);
        // parsing result
        let result: f32 = match commands[1] {
            "\"1-0\";" => 1.0,
            "\"1/2-1/2\";" => 0.5,
            _ => 0.0
        };
        Self {psts, counters, phase, result}
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
    fn slice_error(&self, k: f32, i: usize, ppt: usize) -> f32 {
        self.positions[i*ppt..(i + 1)*ppt].iter()
            .fold(0.0, |err, p| err + (p.result - 1.0 / (1.0 + 10f32.powf(-k * p.eval(&self.params) / 100.0))).powi(2))
    }
}

fn error(k: f32, stuff: &Stuff) -> f32 {
    let ppt: usize = stuff.positions.len() / THREADS;
    let total_error: f32 = scope(|s: &Scope|
        (0..THREADS).map(|i| s.spawn(move || stuff.slice_error(k, i, ppt)))
            .collect::<Vec<ScopedJoinHandle<f32>>>().into_iter()
            .fold(0.0, |err, p| err + p.join().unwrap())
    );
    total_error / stuff.num
}

fn print_psts(params: &[i16; 768]) {
    const PIECES: [&str; 6] = ["pawn", "knight", "bishop", "rook", "queen", "king"];
    const PHASE: [&str; 2] = ["mg", "eg"];
    for i in 0..12 {
        println!("{} {}: {:?}", PHASE[i / 6], PIECES[i % 6], &params[i * 64 .. (i + 1) * 64]);
    }
}

fn main() {
    // LOADING POSITIONS
    let mut stuff: Stuff = Stuff {
        params: [
            [100; 64], [300; 64], [300; 64], [500; 64], [900; 64], [0; 64],
            [100; 64], [300; 64], [300; 64], [500; 64], [900; 64], [0; 64],
        ].concat().try_into().unwrap(),
        positions: Vec::new(),
        num: 0.0,
    };
    let mut time: Instant = Instant::now();
    let mut n: usize = 0;
    let file: File = File::open("set.epd").unwrap_or_else(move |_| {
        println!("Couldn't load file!");
        std::thread::sleep(Duration::from_secs(5));
        std::process::exit(1)
    });
    for line in BufReader::new(file).lines() {
        let pos: Position = Position::from_epd(&line.unwrap());
        n += 1;
        stuff.positions.push(pos);
    }
    let elapsed: u128 = time.elapsed().as_millis();
    println!("loaded {} positions in {} seconds ({}/sec)", n, elapsed as f32 / 1000.0, n * 1000 / elapsed as usize);
    stuff.num = n as f32;

    // OPTIMISING K VALUE
    time = Instant::now();
    let mut k: f32 = INIT_K;
    let mut best_error: f32 = error(k, &stuff);
    let step: f32 = if error(k - STEP_K, &stuff) < error(k + STEP_K, &stuff) {-STEP_K} else {STEP_K};
    loop {
        let new_error: f32 = error(k + step, &stuff);
        if new_error >= best_error {break}
        k += step;
        best_error = new_error;
    }
    println!("optimal k: {:.3}, error: {:.6}, time: {:.2}s", k, best_error, time.elapsed().as_millis() as f32 / 1000.0);
    let k: f32 = k;

    // TEXEL TUNING
    let mut improves: [i16; 768] = [1; 768];
    let mut improved: bool = true;
    let mut count: i32 = 1;
    let mut new_error: f32;
    while improved {
        time = Instant::now();
        improved = false;
        for (i, dir) in improves.iter_mut().enumerate() {
            stuff.params[i] += *dir;
            new_error = error(k, &stuff);
            if new_error < best_error {
                best_error = new_error;
                improved = true;
            } else {
                stuff.params[i] -= 2 * (*dir);
                new_error = error(k, &stuff);
                if new_error < best_error {
                    best_error = new_error;
                    improved = true;
                    *dir = -(*dir);
                } else {
                    stuff.params[i] += *dir;
                }
            }
        }
        println!("epoch {}: {:.2}s, error: {:.6}", count, time.elapsed().as_millis() as f32 / 1000.0, best_error);
        count += 1;
    }
    println!("Finished optimisation.");
    print_psts(&stuff.params);

    // WAIT FOR EXIT
    loop {
        let mut input: String = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        match commands[0] {
            "quit" => std::process::exit(0),
            _ => println!("Unknown command!"),
        }
    }
}
