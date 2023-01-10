mod consts;
mod score;
mod eval;
mod position;

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
    println!("pub const MATERIAL: [S; 5] = {:?};", &data.params[PAWN..=QUEEN]);
    println!("pub const PROGRESS: [S; 5] = {:?};", &data.params[PAWN_PROGRESSION..PAWN_PROGRESSION + 5]);
    println!("pub const KING_RANKS: [S; 8] = {:?};", &data.params[KING_RANKS..KING_RANKS + 8]);
    println!("pub const MAJOR_DEFEND: [S; {MAJOR_PIECES}] = {:?};", &data.params[MAJOR_DEFEND..MAJOR_DEFEND + MAJOR_PIECES]);
    println!("pub const MAJOR_ATTACK: [S; {MAJOR_PIECES}] = {:?};", &data.params[MAJOR_ATTACK..MAJOR_ATTACK + MAJOR_PIECES]);
    println!("pub const PAWN_SHIELD: S = {:?};", &data.params[PAWN_SHIELD]);
    println!("pub const PAWN_PASSED: S = {:?};", &data.params[PAWN_PASSED]);
    println!("pub const KNIGHT_OUTER: S = {:?};", &data.params[KNIGHT_OUTER]);
    println!("pub const ROOK_AGGRESSIVE: S = {:?};", &data.params[ROOK_AGGRESSIVE]);

    // WAIT FOR EXIT
    stdin().read_line(&mut String::new()).expect("parsable");
}
