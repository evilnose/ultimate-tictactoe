use crate::moves::*;
use crate::engine::*;
use crate::engine::eval::*;

// brute force negamax with no pruning
pub fn brute_force_search(depth: u16, pos: &mut Position) -> Score {
    assert!(pos.assert());
    if depth == 0 {
        return basic_eval(pos);
    }

    // check if game is over
    let result = pos.get_result();
    match result {
        GameResult::X_WON => return 100,
        GameResult::O_WON => return -100,
        GameResult::DRAW => return 0,
        GameResult::ONGOING => {}
    }

    let mut max_score = SCORE_NEG_INF;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let score = -brute_force_search(depth - 1, &mut temp);
        if score > max_score {
            max_score = score;
        }
    }

    assert_ne!(max_score, SCORE_NEG_INF);
    return max_score;
}
