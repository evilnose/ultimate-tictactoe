// codinggame more like codinggae amirite
use std::io::{self, BufRead};

extern crate uttt;

use uttt::engine::*;
use uttt::moves::*;

use std::time::{Instant};

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

        if opp_row != -1 {
            // convert index
            let index = (opp_col/3)*9 + (opp_col % 3) + (opp_row/3)*27 + 3*(opp_row %3);
            pos.make_move(index as u8);
        }

        let now = Instant::now();
        let manager = Manager::from_position(pos);
        let res = manager.search_fixed_time(100);

        let elapsed = now.elapsed();
        //eprintln!("elapsed: {} ms. move: {}, eval: {}", elapsed.as_millis(), res.best_move, res.eval);
        let idx = res.best_move;
        let col = ((idx/9) % 3)*3 + (idx % 3);
        let row = ((idx/9) / 3)*3 + (idx % 9)/3;
        pos.make_move(idx);
        println!("{} {} Elapsed: {}, {} ms", row, col, res.eval, elapsed.as_millis());
    }
}
