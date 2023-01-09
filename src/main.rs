mod consts;
mod score;
mod eval;
mod position;

pub use eval::NUM_PARAMS;
use consts::*;
use score::S;
use position::Position;
use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
    thread::{available_parallelism, scope, ScopedJoinHandle},
    time::Instant,
};

// optimising k-value stuff
const K: f32 = 0.4;
const STEP: f32 = 0.001;

// collects all data into one struct
struct Data {
    params: [S; NUM_PARAMS],
    positions: Vec<Position>,
    num: f32,
    size: usize,
}

fn error(k: f32, data: &Data) -> f32 {
    let total: f32 = scope(|s| {
        data.positions
            .chunks(data.size)
            .map(|chunk| s.spawn(|| chunk.iter().map(|pos| pos.err(k, &data.params)).sum()))
            .collect::<Vec<ScopedJoinHandle<f32>>>().into_iter()
            .map(|p| p.join().unwrap_or_default())
            .sum()
    });
    total / data.num
}

fn main() {
    // LOADING POSITIONS
    let mut data = Data {
        params: S::INIT,
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
    println!("positions {:.0} ({}/sec)", data.num, data.num / time.elapsed().as_secs_f32());

    // OPTIMISING K VALUE
    time = Instant::now();
    let step: f32 = (error(K - STEP, &data) - error(K + STEP, &data)).signum() * STEP;
    let (mut k, mut best, mut new): (f32, f32, f32) = (K, error(K, &data), error(K + step, &data));
    while new <= best {
        k += step;
        best = new;
        new = error(k + step, &data);
    }
    println!("time {:.3}s error {best:.6} optimal k = {k:.3}", time.elapsed().as_secs_f32());

    // TEXEL TUNING
    let mut cache: [S; NUM_PARAMS] = [S::ONES; NUM_PARAMS];
    let mut improved: bool = true;
    let mut count = 0;
    while improved {
        time = Instant::now();
        improved = false;
        count += 1;
        for (i, dir) in cache.iter_mut().enumerate() { // iterate over all params
            for j in [false, true] { // iterate over midgame and endgame parts
                data.params[i][j] += dir[j];
                new = error(k, &data);
                if new < best { // cached direction lead to an improvement
                    best = new;
                    improved = true;
                } else {
                    data.params[i][j] -= 2 * dir[j];
                    new = error(k, &data);
                    if new < best { // opposite of cached direction led to an improvement
                        best = new;
                        improved = true;
                        dir[j] = -dir[j];
                    } else { // no improvement
                        data.params[i][j] += dir[j];
                    }
                }
            }
        }
        println!("epoch {count} time {:.3}s error {best:.6}", time.elapsed().as_secs_f32());
    }
    // outputting parameters
    println!("pub const MATERIAL: [S; 7] = {:?}, S(0, 0), S(0, 0)];", &data.params[PAWN..=QUEEN]);
    println!("pub const THREATS: [S; 4] = {:?};", &data.params[5..9]);
    println!("pub const SUPPORTS: [S; 4] = {:?};", &data.params[9..13]);
    println!("pub const CONTROLS: [S; 4] = {:?};", &data.params[13..17]);
    println!("pub const KING_SAFETY: S = {:?};", &data.params[KING_SAFETY]);
    println!("pub const PAWN_SUPPORTS: S = {:?};", &data.params[PAWN_SUPPORTS]);
    println!("pub const PAWN_THREATS: S = {:?};", &data.params[PAWN_THREATS]);
    println!("pub const PAWN_SHIELD: S = {:?};", &data.params[PAWN_SHIELD]);
    println!("pub const PASSED_PAWNS: S = {:?};", &data.params[PASSED_PAWNS]);
    println!("pub const PAWN_PROGRESSION: [S; 6] = {:?};", &data.params[PAWN_PROGRESSION..PAWN_FILES]);
    println!("pub const PAWN_FILES: [S; 8] = {:?};", &data.params[PAWN_FILES..PAWN_FILES + 8]);
    println!("pub const BISHOP_PAIR: S = {:?};", &data.params[BISHOP_PAIR]);

    // WAIT FOR EXIT
    stdin().read_line(&mut String::new()).expect("parsable");
}
