pub mod config;
pub mod eval;
pub mod search;
pub mod utils;

use std::time::Duration;
use std::time::Instant;
use std::thread;

use crate::engine::config::*;
use crate::engine::eval::*;
use crate::engine::search::*;
use crate::moves::*;

// one worker per thread. used for searching
pub struct Worker {
    position: Position,
    eval_fn: EvalFn,

    // context variables during a search; need to call reset
    // function before each search
    nodes_searched: u64,
}

impl Worker {
    pub fn from_position(pos: &Position) -> Worker {
        Worker {
            position: pos.clone(),
            eval_fn: eval, // default to eval; might change later
            nodes_searched: 0,
        }
    }

    pub fn search_till_depth(&self, depth: u16) -> Score {
        self.alpha_beta_dfs(depth, &self.position, SCORE_NEG_INF, SCORE_POS_INF)
    }
    
    pub fn search_free(&self, x_millis: u64, o_millis: u64) -> SearchResult {
        // TODO use the other player's time
        let my_millis = match self.position.to_move {
            Side::X => x_millis,
            Side::O => o_millis,
        };

        //let alloc_time = my_time / (81 - pos)
        let cur_ply = self.position.cur_ply();
        debug_assert!(cur_ply < 81);

        // divide remaining time by remaining moves
        // multiply by something to be a bit more generous
        // very late game
        let alloc_millis: u128;
        if cur_ply > 60 {
            // should be over soon, so take one third
            alloc_millis = std::cmp::max(1000, my_millis as u128 / 3);
        } else {
            // be optimistic and assume the game is over by 60 plies
            alloc_millis = (my_millis as f32 / (65.0 - cur_ply as f32)) as u128;

            // guarantee 5 seconds
            //alloc_millis = std::cmp::max(alloc_millis, 5000);
        }

        eprintln!("GARY: secs remaining: {}; allocated {}", my_millis as f32 /1000.0, alloc_millis as f32 /1000.0);

        let mut moves = self.position.legal_moves();

        let last_best = moves.next().expect("error: no legal moves");
        let mut best = last_best;
        let mut best_score = SCORE_NEG_INF;

        let start_search = Instant::now();

        for depth in 1..=MAX_SEARCH_PLIES {
            let mut move_idx = 1;
            let mut localmoves = moves.clone();
            // search last best move first
            {
                let mut localpos = self.position.clone();
                localpos.make_move(last_best);
                let score = self.alpha_beta_dfs(depth - 1, &localpos, SCORE_NEG_INF, SCORE_POS_INF);
                if score > best_score {
                    best_score = score;
                    // no need to update best since it is already best_move == besc
                }
                localmoves.remove(last_best);
            }

            // search the remaining moves
            for mv in localmoves {
                // check if exceeded time
                if start_search.elapsed().as_millis() >= alloc_millis {
                    eprintln!("GARY: stopping search at depth {} and move {}", depth, move_idx);
                    eprintln!("GARY: actual elapsed: {}", start_search.elapsed().as_secs_f32());
                    return SearchResult {
                        best_move: best,
                        eval: best_score,
                    };
                }
                move_idx += 1;

                let mut localpos = self.position.clone();
                localpos.make_move(mv);
                let score = self.alpha_beta_dfs(depth - 1, &localpos, SCORE_NEG_INF, SCORE_POS_INF);
                if score > best_score {
                    best_score = score;
                    best = mv;
                }
            }
        }

        eprintln!("GARY: stopping search since MAX_SEARCH_PLIES exceeded");
        eprintln!("GARY: actual elapsed: {}", start_search.elapsed().as_secs_f32());
        return SearchResult {
            best_move: best,
            eval: best_score,
        };
    }

    /*
    // alpha-beta iterative deepening search
    fn limited_search(&self, pos: &Position, alpha: Score, beta: Score) -> SearchResult {
        debug_assert!(pos.assert());
    }
    */

    // alpha-beta negamax search using DFS
    // TODO return SearchResult instead
    fn alpha_beta_dfs(&self, depth: u16, pos: &Position, alpha: Score, beta: Score) -> Score {
        debug_assert!(pos.assert());

        // this move has won -- it's terrible for the current side
        // note only the last moved side could have won so only
        // one call to is_won() is made
        // TODO do we need to check for this? would alpha-beta take care of this
        if pos.is_won(pos.to_move.other()) {
            return SCORE_NEG_INF;
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
            let score = -self.alpha_beta_dfs(depth - 1, &mut temp, -beta, -alpha);
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
    let mut best_score = SCORE_LOSS;
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
