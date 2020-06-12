use uttt::moves::*;
use uttt::engine;
mod common;

#[test]
fn startpos() {
    common::setup();
    let mut pos = Position::new();
    assert_eq!(perft(6, &mut pos), 33782544);
}

#[test]
fn draw_in_5() {
    common::setup();
    let move_list = "0, 1, 9, 4, 36, 7, 70, 71, 79, 67, 43, 63, 20, 21,\
                     31, 40, 37, 13, 38, 23, 49, 22, 10, 14, 52, 55, 11,\
                     50, 46, 30, 29, 27, 32, 33, 58, 78, 59, 72, 57";
    let mut pos = Position::from_move_list(move_list);
    assert_eq!(perft(5, &mut pos), 72);
    assert_eq!(perft(6, &mut pos), 0);
}

#[test]
fn early_mid() {
    common::setup();
    let move_list = "0, 3, 27, 4, 36, 5, 46, 13, 37, 12, 28, 14";
    let mut pos = Position::from_move_list(move_list);
    assert_eq!(perft(5, &mut pos), 4876350);
}
