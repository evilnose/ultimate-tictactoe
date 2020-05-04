pub mod eval;
pub mod search;

use crate::moves::*;
use crate::engine::search::*;

type Score = i32;
const SCORE_NEG_INF: i32 = std::i32::MIN;
const score_POS_INF: i32 = std::i32::MAX;

// no time limit; single thread
pub fn best_move(depth: u16, pos: &mut Position) -> (Idx, Score) {
    assert!(depth >= 1);
    let mut best_score = SCORE_NEG_INF;
    let mut best_move = NULL_IDX;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let score = -brute_force_search(depth - 1, &mut temp);
        if score > best_score {
            best_score = score;
            best_move = mov;
        }
    }

    return (best_move, -best_score);
}
