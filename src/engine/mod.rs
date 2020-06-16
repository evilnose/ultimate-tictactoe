pub mod config;
pub mod eval;
pub mod search;

use crate::moves::*;
use crate::engine::search::*;
use crate::engine::eval::*;
use crate::engine::config::*;

// one worker per thread. used for searching
pub struct Worker {
    position: Position,
    eval_fn: EvalFn,
}

impl Worker {
    pub fn from_position(pos: &Position) -> Worker {
        Worker {
            position: pos.clone(),
            eval_fn: eval,  // default to eval; might change later
        }
    }

    pub fn search_till_depth(&self, depth: u16) -> Score {
        self.alpha_beta_search(depth, &self.position, SCORE_NEG_INF, SCORE_POS_INF)
    }

    // alpha-beta negamax search algorithm
    fn alpha_beta_search(&self, depth: u16, pos: &Position, alpha: Score, beta: Score) -> Score {
        debug_assert!(pos.assert());

        // this move has won -- it's terrible for the current side
        // note only the last moved side could have won so only
        // one call to is_won() is made
        // TODO do we need to check for this? would alpha-beta take care of this
        if pos.is_won(pos.to_move.other()) {
            return -std::f32::INFINITY;
        } else if pos.is_drawn() {
            return 0.0;
        } else if depth == 0 {
            let f = self.eval_fn;
            return f(pos);
        }

        let mut alpha = alpha;

        for mov in pos.legal_moves() {
            let mut temp = pos.clone();
            temp.make_move(mov);
            let score = -self.alpha_beta_search(depth - 1, &mut temp, -beta, -alpha);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        return alpha;   
    }
}

pub fn init_engine() {
    init_block_score_table();
}

// no time limit; single thread
pub fn best_move(depth: u16, pos: &Position) -> (Idx, Score) {
    debug_assert!(depth >= 1);
    debug_assert!(!pos.is_over());
    let mut best_score = SCORE_NEG_INF;
    let mut best_move = NULL_IDX;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let worker = Worker::from_position(&temp);
        let score = -worker.search_till_depth(depth - 1);
        if score > best_score {
            best_score = score;
            best_move = mov;
        }
    }
    debug_assert!(best_move != NULL_IDX);
    return (best_move, best_score * side_multiplier(pos.to_move));
}
