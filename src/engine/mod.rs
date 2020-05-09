pub mod eval;
pub mod search;

use crate::moves::*;
use crate::engine::search::*;
use crate::engine::eval::side_multiplier;

type Score = i32;
// don't exceed these pls
const SCORE_NEG_INF: i32 = -10000;
const SCORE_POS_INF: i32 = 10000;

// no time limit; single thread
pub fn best_move(depth: u16, pos: &mut Position) -> (Idx, Score) {
    // TODO maybe debug_assert? probably doesn't matter
    assert!(depth >= 1);
    assert!(!pos.is_over());
    let mut best_score = SCORE_NEG_INF;
    let mut best_move = NULL_IDX;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let score = -alpha_beta_search(depth, &mut temp);
        if score > best_score {
            best_score = score;
            best_move = mov;
        }
    }
    return (best_move, best_score * side_multiplier(pos.to_move));
}
