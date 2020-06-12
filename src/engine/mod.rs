pub mod eval;
pub mod search;

use crate::moves::*;
use crate::engine::search::*;
use crate::engine::eval::*;

pub type Score = f32;
// don't exceed these pls
const SCORE_NEG_INF: f32 = -1000000.0;
const SCORE_POS_INF: f32 = 1000000.0;

pub fn init_engine() {
    init_block_score_table();
}

// no time limit; single thread
pub fn best_move(depth: u16, pos: &mut Position) -> (Idx, Score) {
    assert!(depth >= 1);
    assert!(!pos.is_over());
    let mut best_score = SCORE_NEG_INF;
    let mut best_move = NULL_IDX;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let score = -alpha_beta_search(depth, &mut temp, eval);
        if score > best_score {
            best_score = score;
            best_move = mov;
        }
    }
    debug_assert!(best_move != NULL_IDX);
    return (best_move, best_score * side_multiplier(pos.to_move));
}

// no time limit; single thread
pub fn basic_best_move(depth: u16, pos: &mut Position) -> (Idx, Score) {
    assert!(depth >= 1);
    assert!(!pos.is_over());
    let mut best_score = SCORE_NEG_INF;
    let mut best_move = NULL_IDX;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let score = -alpha_beta_search(depth, &mut temp, basic_eval);
        if score > best_score {
            best_score = score;
            best_move = mov;
        }
    }
    debug_assert!(best_move != NULL_IDX);
    return (best_move, best_score * side_multiplier(pos.to_move));
}
