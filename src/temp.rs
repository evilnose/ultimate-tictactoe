// mod moves;
use uttt::moves::*;
use uttt::engine;
use std::path::Path;
use engine::*;
use std::time::Instant;

fn from_file(fname: &str, to_move: Side, auto_side: bool) -> Position {
    let s = std::fs::read_to_string(Path::new(fname)).unwrap();
    return Position::from_compact_board(&s[..], to_move, auto_side);
}

fn main() {
    // static BOARD: &str =
    // "O..|XX.|...\n\
    //  ...|.X.|...\n\
    //  ...|X.O|...\n\
    //  -----------\n\
    //  ...|...|...\n\
    //  O..|...|...\n\
    //  ...|...|...\n\
    //  -----------\n\
    //  ...|...|...\n\
    //  ...|...|...\n\
    //  ...|...|... 0";
    // let mut pos = Position::from_board(BOARD);
    // println!("{}", pos.legal_moves().size());
    // println!("{}", pos.to_pretty_board());
    init_moves();
    engine::init_engine();
    let pos = from_file("file.txt", Side::O, false);

    println!("{}", pos.to_pretty_board());
    let now = Instant::now();
    let manager = Manager::from_position(pos);
    let res = manager.search_fixed_time(80);

    let elapsed = now.elapsed();
    eprintln!("elapsed: {} ms. move: {}, eval: {}", elapsed.as_millis(), res.best_move, res.eval);
    /*
    let tup = engine::best_move(11, &mut pos);
    println!("{}", tup.0);
    */
    // perft_with_progress(10, &mut pos);
    // println!("{}", perft(8, &mut pos));
}
