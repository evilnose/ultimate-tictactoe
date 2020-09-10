// codingame more like codinggae amirite
use std::io::{self, BufRead};
use std::time::{Instant};
use rand::SeedableRng;
use rand::rngs::SmallRng;

extern crate uttt;

use uttt::engine::*;
use uttt::moves::*;
use uttt::engine::mcts::*;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn next_line() -> String {
    io::stdin()
        .lock()
        .lines()
        .next()
        .expect("there was no next line")
        .expect("the line could not be read")
}

fn main() {
    let c: f32 = match std::env::args().nth(1) {
        Some(val) => val.parse().expect(&format!("Could not parse c value '{}'", val)[..]),
        None => 0.85,
    };
    init_moves();
    init_engine();
    let mut pos = Position::new();
    loop {
        let line = next_line();
        let inputs = line.split(" ").collect::<Vec<_>>();
        let opp_row = parse_input!(inputs[0], i32);
        let opp_col = parse_input!(inputs[1], i32);

        let line = next_line();
        let valid_action_count = parse_input!(line, i32);
        for _i in 0..valid_action_count as usize {
            next_line();
        }

        if opp_row == -1 {
            // hardcoded center move
            println!("4 4 auto");
            pos.make_move(40);
            continue;
        }
        let index = (opp_col/3)*9 + (opp_col % 3) + (opp_row/3)*27 + 3*(opp_row %3);
        pos.make_move(index as u8);

        let now = Instant::now();
        //let manager = Manager::from_position(pos);
        //let res = manager.search_fixed_time(100);
        //let idx = res.best_move;
        //let rng = SmallRng::seed_from_u64(12345);
        let rng = SmallRng::from_entropy();
        let mut mcts = MCTSWorker::new(pos, c, rng);
        let (res, n_rollouts) = mcts.go(100);
        let idx = res.best_move;
        let eval = res.value;

        let elapsed = now.elapsed();
        //eprintln!("elapsed: {} ms. move: {}, eval: {}", elapsed.as_millis(), res.best_move, res.eval);
        let col = ((idx/9) % 3)*3 + (idx % 3);
        let row = ((idx/9) / 3)*3 + (idx % 9)/3;
        pos.make_move(idx);
        println!(
            "{} {} {}/{}",
            row,
            col,
            eval,
            n_rollouts,
        );
        eprintln!("actual elapsed: {} ms", elapsed.as_millis());
        /*
        for e in mcts.pv() {
            eprintln!("move {}; value {}", e.best_move, e.value);
        }
        */
    }
}
