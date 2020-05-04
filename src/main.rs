extern crate bit_vec;

// mod moves;
use uttt::moves::{Position, perft_with_progress};

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
    let mut pos = Position::new();
    perft_with_progress(10, &mut pos);
}
