use std::{fs::File, io::{BufRead, BufReader}, time::{Duration, Instant}};

// initial parameters are PeSTO values
static mut PARAMS: [i16; 768] = [
    82, 82, 82, 82, 82, 82, 82, 82, 180, 216, 143, 177, 150, 208, 116, 71, 76, 89, 108, 113, 147, 138, 107, 62, 68, 95, 88, 103, 105, 94, 99, 59, 55, 80, 77, 94, 99, 88, 92, 57, 56, 78, 78, 72, 85, 85, 115, 70, 47, 81, 62, 59, 67, 106, 120, 60, 82, 82, 82, 82, 82, 82, 82, 82,
    170, 248, 303, 288, 398, 240, 322, 230, 264, 296, 409, 373, 360, 399, 344, 320, 290, 397, 374, 402, 421, 466, 410, 381, 328, 354, 356, 390, 374, 406, 355, 359, 324, 341, 353, 350, 365, 356, 358, 329, 314, 328, 349, 347, 356, 354, 362, 321, 308, 284, 325, 334, 336, 355, 323, 318, 232, 316, 279, 304, 320, 309, 318, 314,
    336, 369, 283, 328, 340, 323, 372, 357, 339, 381, 347, 352, 395, 424, 383, 318, 349, 402, 408, 405, 400, 415, 402, 363, 361, 370, 384, 415, 402, 402, 372, 363, 359, 378, 378, 391, 399, 377, 375, 369, 365, 380, 380, 380, 379, 392, 383, 375, 369, 380, 381, 365, 372, 386, 398, 366, 332, 362, 351, 344, 352, 353, 326, 344,
    509, 519, 509, 528, 540, 486, 508, 520, 504, 509, 535, 539, 557, 544, 503, 521, 472, 496, 503, 513, 494, 522, 538, 493, 453, 466, 484, 503, 501, 512, 469, 457, 441, 451, 465, 476, 486, 470, 483, 454, 432, 452, 461, 460, 480, 477, 472, 444, 433, 461, 457, 468, 476, 488, 471, 406, 458, 464, 478, 494, 493, 484, 440, 451,
    997, 1025, 1054, 1037, 1084, 1069, 1068, 1070, 1001, 986, 1020, 1026, 1009, 1082, 1053, 1079, 1012, 1008, 1032, 1033, 1054, 1081, 1072, 1082, 998, 998, 1009, 1009, 1024, 1042, 1023, 1026, 1016, 999, 1016, 1015, 1023, 1021, 1028, 1022, 1011, 1027, 1014, 1023, 1020, 1027, 1039, 1030, 990, 1017, 1036, 1027, 1033, 1040, 1022, 1026, 1024, 1007, 1016, 1035, 1010, 1000, 994, 975,
    -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20, 6, 22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51, -14, -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8, -28, 24, 14,
    94, 94, 94, 94, 94, 94, 94, 94, 272, 267, 252, 228, 241, 226, 259, 281, 188, 194, 179, 161, 150, 147, 176, 178, 126, 118, 107, 99, 92, 98, 111, 111, 107, 103, 91, 87, 87, 86, 97, 93, 98, 101, 88, 95, 94, 89, 93, 86, 107, 102, 102, 104, 107, 94, 96, 87, 94, 94, 94, 94, 94, 94, 94, 94,
    223, 243, 268, 253, 250, 254, 218, 182, 256, 273, 256, 279, 272, 256, 257, 229, 257, 261, 291, 290, 280, 272, 262, 240, 264, 284, 303, 303, 303, 292, 289, 263, 263, 275, 297, 306, 297, 298, 285, 263, 258, 278, 280, 296, 291, 278, 261, 259, 239, 261, 271, 276, 279, 261, 258, 237, 252, 230, 258, 266, 259, 263, 231, 217,
    283, 276, 286, 289, 290, 288, 280, 273, 289, 293, 304, 285, 294, 284, 293, 283, 299, 289, 297, 296, 295, 303, 297, 301, 294, 306, 309, 306, 311, 307, 300, 299, 291, 300, 310, 316, 304, 307, 294, 288, 285, 294, 305, 307, 310, 300, 290, 282, 283, 279, 290, 296, 301, 288, 282, 270, 274, 288, 274, 292, 288, 281, 292, 280,
    525, 522, 530, 527, 524, 524, 520, 517, 523, 525, 525, 523, 509, 515, 520, 515, 519, 519, 519, 517, 516, 509, 507, 509, 516, 515, 525, 513, 514, 513, 511, 514, 515, 517, 520, 516, 507, 506, 504, 501, 508, 512, 507, 511, 505, 500, 504, 496, 506, 506, 512, 514, 503, 503, 501, 509, 503, 514, 515, 511, 507, 499, 516, 492,
    927, 958, 958, 963, 963, 955, 946, 956, 919, 956, 968, 977, 994, 961, 966, 936, 916, 942, 945, 985, 983, 971, 955, 945, 939, 958, 960, 981, 993, 976, 993, 972, 918, 964, 955, 983, 967, 970, 975, 959, 920, 909, 951, 942, 945, 953, 946, 941, 914, 913, 906, 920, 920, 913, 900, 904, 903, 908, 914, 893, 931, 904, 916, 895,
    -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45, 44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23, 16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
];

static mut IMPROVES: [i16; 768] = [1; 768];
static mut POSITIONS: Vec<Position> = Vec::new();
static mut NUM: usize = 0;

const PHASE_VALS: [i16; 7] = [0, 1, 1, 2, 4, 0, 0];
const TPHASE: i32 = 24;

#[derive(Debug, Default)]
struct Position {
    psts: [[usize; 16]; 2],
    counters: [usize; 2],
    phase: i16,
    result: f64,
}

impl Position {
    unsafe fn eval(&self) -> i16 {
        let p = self.phase as i32;
        let mut mg = 0;
        let mut eg = 0;
        for i in 0..self.counters[0] {
            mg += PARAMS[self.psts[0][i]] as i32;
            eg += PARAMS[self.psts[0][i] + 384] as i32;
        }
        for i in 0..self.counters[1] {
            mg -= PARAMS[self.psts[1][i]] as i32;
            eg -= PARAMS[self.psts[1][i] + 384] as i32;
        }
        ((p * mg + (TPHASE - p) * eg) / TPHASE) as i16
    }
}

fn parse_epd(s: &str) -> Position {
    let commands: Vec<&str> = s.split("c9").map(|v| v.trim()).collect();

    // parsing fen
    let mut phase = 0;
    let mut psts = [[0; 16]; 2];
    let mut counters = [0, 0];
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
    let result = match commands[1] {
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

unsafe fn calculate_error(k: f64) -> f64 {
    let mut error = 0.0;
    for pos in &POSITIONS {
        error += (pos.result - sigmoid(k, pos.eval() as f64 / 100.0)).powi(2);
    }
    error / NUM as f64
}

unsafe fn optimise_k(mut initial_guess: f64, step_size: f64) -> f64 {
    let mut best_error = calculate_error(initial_guess);
    let step = if calculate_error(initial_guess - step_size) < calculate_error(initial_guess + step_size) {
        -step_size
    } else {
        step_size
    };
    loop {
        let new_error = calculate_error(initial_guess + step);
        if new_error < best_error {
            initial_guess += step;
            best_error = new_error;
        } else {
            break;
        }
    }
    initial_guess
}

unsafe fn print_psts() {
    const PIECES: [&str; 6] = ["pawn", "knight", "bishop", "rook", "queen", "king"];
    const PHASE: [&str; 2] = ["mg", "eg"];
    for i in 0..12 {
        println!("{} {}: {:?}", PHASE[i / 6], PIECES[i - 6 * (i >= 6) as usize], &PARAMS[i * 64 .. (i + 1) * 64]);
    }
}

fn main() {
    unsafe {

    // LOADING POSITIONS
    let mut time = Instant::now();
    NUM = 0;

    let file = match File::open("set.epd") {
        Ok(f) => f,
        _ => {
            println!("Couldn't load file!");
            std::thread::sleep(Duration::from_secs(5));
            return
        }
    };

    for line in BufReader::new(file).lines(){
        let pos = parse_epd(&line.unwrap());
        NUM += 1;
        POSITIONS.push(pos);
    }
    let elapsed = time.elapsed().as_millis();
    println!("loaded {} positions in {} seconds ({}/sec)", NUM, elapsed as f64 / 1000.0, NUM * 1000 / elapsed as usize);

    // OPTIMISING K VALUE
    time = Instant::now();
    let k = optimise_k(0.4, 0.01);
    let mut best_error = calculate_error(k);
    println!("optimal k: {:.6}, error: {:.6}, time: {:.2}s", k, best_error, time.elapsed().as_millis() as f64 / 1000.0);

    // TEXEL TUNING
    let mut improved = true;
    let mut count = 1;
    while improved {
        time = Instant::now();
        improved = false;
        for (i, param) in PARAMS.iter_mut().enumerate() {
            *param += IMPROVES[i];
            let new_error = calculate_error(k);
            if new_error < best_error {
                best_error = new_error;
                improved = true;
            } else {
                *param -= 2 * IMPROVES[i];
                let new_error2 = calculate_error(k);
                if new_error2 < best_error {
                    best_error = new_error2;
                    improved = true;
                    IMPROVES[i] = -IMPROVES[i];
                } else {
                    *param += IMPROVES[i];
                }
            }
        }
        println!("epoch {}: {:.2}s, error: {:.6}", count, time.elapsed().as_millis() as f64 / 1000.0, best_error);
        count += 1;
    }
    println!("Finished optimisation.");
    print_psts();

    // wait to exit
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        match commands[0] {
            "quit" => std::process::exit(0),
            _ => println!("Unknown command!"),
        }
    }
    }
}
