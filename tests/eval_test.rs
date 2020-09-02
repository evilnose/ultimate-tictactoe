use uttt::engine::*;
use uttt::moves::*;

mod common;

#[test]
fn basic_search() {
    common::setup();
    let mut pos = Position::new();
    assert_eq!(perft(6, &mut pos), 33782544);
}

#[test]
fn stupid_search() {
    common::setup();
    let pos = Position::from_move_list(
        "36, 0, 2, 18, 4, 37, 15, 55, 12, 29, 19, 11, 25, 66, 32, 48, 31,\
    40, 39, 30, 35, 74, 24, 58, 42, 61, 63, 5, 53, 80, 77, 45, 6, 14, 50, 47, 23, 46, 9, 75",
    );
    println!("{}", pos.to_pretty_board());
    panic!("Fix best_move first. Implement generic termination function");
    //let tup = best_move(5, &mut pos);
    // assert!(tup.1 > 0.0);
    //let tup = best_move(6, &mut pos);
    // assert!(tup.1 > 0.0);
}

#[test]
fn temp() {
    common::setup();
    // TODo make this private
    let a = eval::eval_block(0b111111111, 0b000000000);
    println!("{}", a);
}
