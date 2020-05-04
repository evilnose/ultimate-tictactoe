use uttt::engine::*;
use uttt::moves::*;

#[test]
fn basic_search() {
    let mut pos = Position::new();
    assert_eq!(perft(6, &mut pos), 33782544);
}
