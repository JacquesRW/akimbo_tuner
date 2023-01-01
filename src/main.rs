use std::{fs::File, io::{BufRead, BufReader}, time::{Duration, Instant}};

const PHASE_VALS: [i16; 7] = [0, 1, 1, 2, 4, 0, 0];
const TPHASE: i32 = 24;

#[derive(Debug, Default)]
struct Position {
    psts: [[usize; 16]; 2],
    counters: [usize; 2],
    phase: i16,
    result: f64,
}

struct Stuff {
    params: [i16; 768],
    positions: Vec<Position>,
    num: f64,
}

impl Position {
    fn eval(&self, params: &[i16; 768]) -> i16 {
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
        ((p * mg as i32 + (TPHASE - p) * eg as i32) / TPHASE) as i16
    }
}

fn parse_epd(s: &str) -> Position {
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
    let result: f64 = match commands[1] {
        "\"1-0\";" => 1.0,
        "\"1/2-1/2\";" => 0.5,
        _ => 0.0
    };

    Position {psts, counters, phase, result}
}

#[inline(always)]
fn sigmoid(k: f64, x: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(- k * x))
}

fn calculate_error(k: f64, stuff: &Stuff) -> f64 {
    let mut error: f64 = 0.0;
    for pos in &stuff.positions {
        error += (pos.result - sigmoid(k, pos.eval(&stuff.params) as f64 / 100.0)).powi(2);
    }
    error / stuff.num
}

fn optimise_k(mut initial_guess: f64, step_size: f64, stuff: &Stuff) -> f64 {
    let mut best_error: f64 = calculate_error(initial_guess, stuff);
    let step: f64 = if calculate_error(initial_guess - step_size, stuff) < calculate_error(initial_guess + step_size, stuff) {
        -step_size
    } else {
        step_size
    };
    loop {
        let new_error: f64 = calculate_error(initial_guess + step, stuff);
        if new_error < best_error {
            initial_guess += step;
            best_error = new_error;
        } else {
            break;
        }
    }
    initial_guess
}

fn print_psts(params: &[i16; 768]) {
    const PIECES: [&str; 6] = ["pawn", "knight", "bishop", "rook", "queen", "king"];
    const PHASE: [&str; 2] = ["mg", "eg"];
    for i in 0..12 {
        println!("{} {}: {:?}", PHASE[i / 6], PIECES[i % 6], &params[i * 64 .. (i + 1) * 64]);
    }
}

fn main() {
    let mut stuff: Stuff = Stuff {
        params: [
            [100; 64], [300; 64], [300; 64], [500; 64], [900; 64], [0; 64],
            [100; 64], [300; 64], [300; 64], [500; 64], [900; 64], [0; 64]
        ].concat().try_into().unwrap(),
        positions: Vec::new(),
        num: 0.0,
    };

    // LOADING positions
    let mut time: Instant = Instant::now();
    let mut n: usize = 0;

    let file: File = match File::open("set.epd") {
        Ok(f) => f,
        _ => {
            println!("Couldn't load file!");
            std::thread::sleep(Duration::from_secs(5));
            return
        }
    };

    for line in BufReader::new(file).lines(){
        let pos: Position = parse_epd(&line.unwrap());
        n += 1;
        stuff.positions.push(pos);
    }
    let elapsed: u128 = time.elapsed().as_millis();
    println!("loaded {} positions in {} seconds ({}/sec)", n, elapsed as f64 / 1000.0, n * 1000 / elapsed as usize);

    stuff.num = n as f64;

    // OPTIMISING K VALUE
    time = Instant::now();
    let k: f64 = optimise_k(0.4, 0.001, &stuff);
    let mut best_error: f64 = calculate_error(k, &stuff);
    println!("optimal k: {:.3}, error: {:.6}, time: {:.2}s", k, best_error, time.elapsed().as_millis() as f64 / 1000.0);

    // stores the direction of change in value that last caused an improvement
    // in error, based on assumption reverting a change is unlikely to help, to
    // keep moving value in the same direction is better - small peerformance improvement
    let mut improves: [i16; 768] = [1; 768];

    // TEXEL TUNING
    let mut improved: bool = true;
    let mut count: i32 = 1;
    while improved {
        time = Instant::now();
        improved = false;
        for (i, dir) in improves.iter_mut().enumerate() {
            stuff.params[i] += *dir;
            let new_error: f64 = calculate_error(k, &stuff);
            if new_error < best_error {
                best_error = new_error;
                improved = true;
            } else {
                stuff.params[i] -= 2 * (*dir);
                let new_error2: f64 = calculate_error(k, &stuff);
                if new_error2 < best_error {
                    best_error = new_error2;
                    improved = true;
                    *dir = -(*dir);
                } else {
                    stuff.params[i] += *dir;
                }
            }
        }
        println!("epoch {}: {:.2}s, error: {:.6}", count, time.elapsed().as_millis() as f64 / 1000.0, best_error);
        count += 1;
    }
    println!("Finished optimisation.");
    print_psts(&stuff.params);

    // wait to exit
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
