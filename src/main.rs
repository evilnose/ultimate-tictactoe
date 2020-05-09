use uttt::engine::best_move;
use uttt::moves::*;
use std::io::{self, BufRead, Stdin};
use std::env;

// TODO duplicate code in interface.rs
fn next_line(stdin: &mut Stdin) -> String {
    stdin
        .lock()
        .lines()
        .next()
        .expect("there was no next line")
        .expect("the line could not be read")
}

fn main() {
    let mut stdin = io::stdin();
    let mut pos = Position::new();
    let mut depth: i32 = 5;

    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        panic!("usage: ./main [depth]");
    }

    if args.len() == 2 {
        depth = args[1].parse().expect("depth need to be an integer");
        if depth <= 0 {
            panic!("depth needs to be > 0");
        }
    }

    loop {
        let line = next_line(&mut stdin);
        let mov: i32 = line.parse().expect("expected i32");
        if mov != -1 {
            assert!(mov >= 0);
            pos.make_move(mov as Idx);
        }
        let best = best_move(depth as u16, &mut pos);
        println!("{}", best.0);
    }
}
