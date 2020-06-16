use crate::moves::*;
use crate::engine::config::*;
use crate::engine::eval::*;

// brute force negamax with no pruning
pub fn brute_force_search(depth: u16, pos: &mut Position, eval_fn: EvalFn) -> Score {
    debug_assert!(pos.assert());

    // this move has won -- it's terrible for the current side
    // note only the last moved side could have won so only
    // one call to is_won() is made
    if pos.is_won(pos.to_move.other()) {
        return -1000.0;
    } else if pos.is_drawn() {
        return 0.0;
    }

    if depth == 0 {
        return eval_fn(pos);
    }

    let mut max_score = SCORE_NEG_INF;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let score = -brute_force_search(depth - 1, &mut temp, eval_fn);
        if score > max_score {
            max_score = score;
        }
    }

    assert_ne!(max_score, SCORE_NEG_INF);
    return max_score;
}
