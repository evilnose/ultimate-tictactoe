use crate::moves::*;
use crate::engine::*;
use crate::engine::eval::*;

// brute force negamax with no pruning
pub fn brute_force_search(depth: u16, pos: &mut Position) -> Score {
    assert!(pos.assert());
    if depth == 0 {
        return basic_eval(pos);
    }

    let end_score = end_check(pos);
    if end_score != -1 {
        // game is over
        return end_score;
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
