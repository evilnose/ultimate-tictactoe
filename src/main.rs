extern crate bit_vec;
extern crate hex;

// mod moves;
use uttt::moves::Position;

fn main() {
    static BOARD: &str =
    "O..|XX.|...\n\
     ...|.X.|...\n\
     ...|X.O|...\n\
     -----------\n\
     ...|...|...\n\
     O..|...|...\n\
     ...|...|...\n\
     -----------\n\
     ...|...|...\n\
     ...|...|...\n\
     ...|...|... 0";
    let mut pos = Position::from_board(BOARD);
    println!("{}", pos.legal_moves().size());
    println!("{}", pos.board_repr());
}
