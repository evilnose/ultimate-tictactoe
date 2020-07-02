extern crate bit_vec;

// mod moves;
use uttt::moves::{Position, perft, perft_with_progress, init_moves};
use uttt::engine;

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
    let mut pos = Position::new();
    let tup = engine::best_move(11, &mut pos);
    println!("{}", tup.0);
    // perft_with_progress(10, &mut pos);
    // println!("{}", perft(8, &mut pos));
}
