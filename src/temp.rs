// mod moves;
use uttt::moves::{Position, perft, perft_with_progress, init_moves};
use uttt::engine;
use std::path::Path;
use uttt::format;

fn from_file(fname: &str) -> Position {
    let s = std::fs::read_to_string(Path::new(fname)).unwrap();
    return Position::from_compact_board(&s[..]);
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
    let mut pos = Position::from_move_list(
        "10, 16, 64, 12, 28, 11, 19, 9, 1, 17, 73, 13, 37, 30, 35, 75, 32, 46, 66, 34, 71, 79, 68, 48, 33, 55, 5, 52, 69, 57, 31, 39, 29, 21, 50, 47, 23, 51, 59, 49, 44, 78, 62, 80, 2"
    );
    for mov in pos.legal_moves() {
        println!("{}", mov);
    }
    /*
    let tup = engine::best_move(11, &mut pos);
    println!("{}", tup.0);
    */
    // perft_with_progress(10, &mut pos);
    // println!("{}", perft(8, &mut pos));
}
