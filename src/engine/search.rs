use crate::moves::*;
use crate::engine::*;
use crate::engine::eval::*;

pub fn alpha_beta_search(depth: u16, pos: &mut Position, eval: EvalFn) -> Score {
    return alpha_beta(depth, pos, SCORE_NEG_INF, SCORE_POS_INF, eval);
}

// alpha-beta negamax search algorithm
fn alpha_beta(depth: u16, pos: &mut Position, alpha: Score, beta: Score, eval_fn: EvalFn) -> Score {
    debug_assert!(pos.assert());

    // this move has won -- it's terrible for the current side
    // note only the last moved side could have won so only
    // one call to is_won() is made
    if pos.is_won(pos.to_move.other()) {
        return -std::f32::INFINITY;
    } else if pos.is_drawn() {
        return 0.0;
    } else if depth == 0 {
        return eval_fn(pos);
    }

    let mut alpha = alpha;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let score = -alpha_beta(depth - 1, &mut temp, -beta, -alpha, eval_fn);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    return alpha;   
}

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
