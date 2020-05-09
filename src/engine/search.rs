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

pub fn alpha_beta_search(depth: u16, pos: &mut Position) -> Score {
    let mut alpha: i32 = SCORE_NEG_INF;
    let mut beta: i32 = SCORE_POS_INF;
    return alpha_beta(depth, pos, &mut alpha, &mut beta);
}

// alpha-beta negamax search algorithm
fn alpha_beta(depth: u16, pos: &mut Position, alpha: &mut i32, beta: &mut i32) -> Score {
    assert!(pos.assert());
    if depth == 0 {
        return basic_eval(pos);
    }

    let end_score = end_check(pos);
    if end_score != -1 {
        // game is over
        return *beta;
    }

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        *alpha *= -1;
        *beta *= -1;
        let score = -alpha_beta(depth - 1, &mut temp, beta, alpha);
        *alpha *= -1;
        *beta *= -1;
        if score > *beta {
            return *beta;
        }
        if score > *alpha {
            *alpha = score;
        }
    }
    return *alpha;   
}
