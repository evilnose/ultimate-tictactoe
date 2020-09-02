pub mod config;
pub mod eval;
pub mod utils;
pub mod mcts;

use std::time::{Duration};
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::fmt;

use crate::engine::config::*;
use crate::engine::eval::*;
use crate::engine::utils::*;
use crate::moves::*;

// used to break out of recursion
struct StopSearch;

impl fmt::Debug for StopSearch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The search has been signaled to stop. This is used internally by alpha_beta_dfs()")
    }
}

// keeps track of the current search state, e.g. node count, time,
// etc. Used across alpha_beta_dfs
pub struct SearchResult {
    pub best_move: Idx,
    pub eval: Score,
}

#[derive(Copy, Clone)]
pub struct Manager {
    position: Position,
}

// NOTE for now Manager is synchrnous, but we may wish to make it async
// later.
impl Manager {
    pub fn from_position(pos: Position) -> Manager {
        Manager {
            position: pos,
        }
    }

    fn search_fixed_time_inner(&self, stop_search: Arc<AtomicBool>) -> SearchResult {
        let moves = self.position.legal_moves();
        let n_moves = moves.size();
        // moves to explore before going parellel
        let till_parallel = std::cmp::min(n_moves / 2, 4);

        let mut best = moves.peek();
        debug_assert!(best != NULL_IDX);
        let mut best_score = SCORE_NEG_INF;

        for depth in 4..=MAX_SEARCH_PLIES {
            let mut moves_copy = moves;
            // search last best move first
            {
                moves_copy.remove(best);

                let mut localpos = self.position;
                localpos.make_move(best);
                let localstop = Arc::clone(&stop_search);
                let worker = Worker::new(self.position, localstop);
                let result = worker.alpha_beta_dfs(depth - 1, localpos, SCORE_NEG_INF, -best_score);
                let score = match result {
                    Ok(sc) => -sc,
                    Err(_) => return SearchResult {
                        eval: best_score,
                        best_move: best,
                    }
                };
                if score > best_score {
                    best_score = score;
                    // don't need to update best
                }
            }

            let mut move_idx = 1;  // already explored one move
            while move_idx < till_parallel {
                let mov = moves_copy.next().unwrap();
                let mut localpos = self.position;
                localpos.make_move(mov);

                let localstop = Arc::clone(&stop_search);
                let worker = Worker::new(self.position, localstop);
                let result = worker.alpha_beta_dfs(depth - 1, localpos, SCORE_NEG_INF, -best_score);
                let score = match result {
                    Ok(sc) => -sc,
                    Err(_) => return SearchResult {
                        eval: best_score,
                        best_move: best,
                    }
                };
                if score > best_score {
                    best_score = score;
                    best = mov;
                }
                move_idx += 1;
            }

            let mut handles = Vec::<JoinHandle<Result<Score, StopSearch>>>::new();
            let mut rem_moves = Vec::<Idx>::new();

            // search the remaining moves in parallel
            while move_idx < n_moves {
                let mov = moves_copy.next().unwrap();
                let mut localpos = self.position;
                localpos.make_move(mov);

                let localstop = Arc::clone(&stop_search);
                let handle = std::thread::spawn(move || {
                    let worker = Worker::new(localpos, localstop);
                    return worker.alpha_beta_dfs(depth - 1, localpos, SCORE_NEG_INF, -best_score);
                });
                handles.push(handle);

                rem_moves.push(mov);
                move_idx += 1;
            }

            let mut i = 0;
            let mut stop_now = false;
            for handle in handles {
                let res = handle.join().unwrap();
                let mov = rem_moves[i];
                match res {
                    Ok(score) => {
                        let score = -score;
                        if score > best_score {
                            best_score = score;
                            best = mov;
                        }
                    },
                    Err(_) => {
                        stop_now = true;
                    },
                }
                i += 1;
            }
            if stop_now {
                break;
            }
            eprintln!("depth {}, best {}, eval {}", depth, best, best_score);
        }
        return SearchResult{
            eval: best_score,
            best_move: best,
        };
    }

    pub fn search_fixed_time(&self, alloc_millis: u64) -> SearchResult {
        // flag to stop search
        let stop_search = Arc::new(AtomicBool::new(false));

        // expect something reasonable
        assert!(alloc_millis > 30);

        let localstop = Arc::clone(&stop_search);
        // HACK. perhaps better is have a function that is not a member of Manager
        // but instead takes a position
        let me = self.clone();
        let handle = std::thread::spawn(move || {
            return me.search_fixed_time_inner(localstop);
        });

        // wait
        thread::sleep(Duration::from_millis(alloc_millis - 25));

        // tell threads to stop
        stop_search.swap(true, Ordering::Relaxed);
        
        let result = handle.join().unwrap();
        return result;
    }
    
    pub fn search_free(&mut self, x_millis: u64, o_millis: u64) {
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

        eprintln!("NOTE: secs remaining: {}; allocated {}", my_millis as f32 / 1000.0, alloc_millis as f32 / 1000.0);
        self.search_fixed_time(alloc_millis as u64);
    }
}

// one worker per thread. used for searching
struct Worker {
    position: Position,
    eval_fn: EvalFn,
    stop: Arc<AtomicBool>,
}

impl Worker {
    // note: takes ownership of tx and stop, so need to make clone
    pub fn new(pos: Position, stop: Arc<AtomicBool>) -> Worker {
        Worker {
            position: pos,
            eval_fn: eval, // default to eval; might change later
            stop: stop,
        }
    }

    /*
    // alpha-beta iterative deepening search
    fn limited_search(&self, pos: &Position, alpha: Score, beta: Score) -> SearchResult {
        debug_assert!(pos.assert());
    }
    */

    // alpha-beta negamax search using DFS
    // TODO return SearchResult instead
    fn alpha_beta_dfs(&self, depth: u16, pos: Position, alpha: Score, beta: Score) -> Result<Score, StopSearch> {
        debug_assert!(pos.assert());

        // this move has won -- it's terrible for the current side
        // note only the last moved side could have won so only
        // one call to is_won() is made
        // TODO do we need to check for this? would alpha-beta take care of this
        if pos.is_won(pos.to_move.other()) {
            //state.nodes_searched += 1;
            return self.check_time(SCORE_LOSS);
        } else if pos.is_drawn() {
            //state.nodes_searched += 1;
            // NOTE that one side could still be considered won in some rulesets by comparing
            // the total number of blocks occupied
            let mult = codingame_drawn(&pos);
            return self.check_time(mult * side_multiplier(pos.to_move) * SCORE_WIN);
        } else if depth == 0 {
            //state.nodes_searched += 1;
            let f = self.eval_fn;
            let my_1occ = self.position.get_1occ(self.position.to_move);
            let their_1occ = self.position.get_1occ(self.position.to_move.other());
            return self.quiesce_search(pos, my_1occ, their_1occ, f);
        }

        let moves = pos.legal_moves();
        let mut alpha = alpha;
        /*
        let DROP_CUTOFF = 30;
        if moves.size() >= DROP_CUTOFF {
            let indices = (0..DROP_CUTOFF).collect::<Vec<_>>();
            
        }*/
        /*
        let my_1occ = pos.get_1occ(pos.to_move);
        let captures = moves.intersect(my_1occ);
        let moves = moves.subtract(captures);
        for mov in captures {
            let mut temp = pos.clone();
            temp.make_move(mov);
            let score = -self.alpha_beta_dfs(depth - 1, temp, -beta, -alpha, state)?;
            if score >= beta {
                return Ok(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }
        */

        for mov in moves {
            let mut temp = pos.clone();
            temp.make_move(mov);
            let score = -self.alpha_beta_dfs(depth - 1, temp, -beta, -alpha)?;
            if score >= beta {
                return Ok(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }
        return Ok(alpha);
    }

    // called when a leaf node is reached. If timed out, return Err(StopSearch). Otherwise
    // return the given eval wrapped in Result
    #[inline(always)]
    fn check_time(&self, eval: Score) -> Result<Score, StopSearch> {
        if self.stop.load(Ordering::Relaxed) {
            return Err(StopSearch);
        }
        return Ok(eval);
    }

    // my_1occ is the occupancy of moves I can make to capture a block.
    // alpha/beta is not used for now since the search space is assumed to be small
    #[inline(always)]
    fn quiesce_search(&self, pos: Position, mut my_1occ: Moves, mut their_1occ: Moves, eval_fn: EvalFn) -> Result<Score, StopSearch> {
        let captures = pos.legal_moves().intersect(my_1occ);
        if captures.size() != 0 {
            let mut best = SCORE_NEG_INF;
            for mov in captures {
                let mut temp = pos.clone();
                temp.make_move(mov);

                my_1occ.remove(mov);
                their_1occ.remove(mov);
                let score = -self.quiesce_search(temp, their_1occ, my_1occ, eval_fn)?;
                if score > best {
                    best = score;
                }
            }
            return self.check_time(best);
        } else {
            return self.check_time(eval_fn(&pos));
        }
    }
}

pub fn init_engine() {
    init_block_score_table();
    init_natural_log_table();
}

/*
// no time limit; single thread
pub fn best_move(depth: u16, pos: &Position) -> (Idx, Score) {
    debug_assert!(depth >= 1);
    debug_assert!(!pos.is_over());
    let mut best_score = SCORE_NEG_INF;
    let mut best_move = NULL_IDX;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        let mut worker = Worker::from_position(temp);
        let score = -worker.search_till_depth(depth - 1);
        if score > best_score {
            best_score = score;
            best_move = mov;
        }
    }
    debug_assert!(best_move != NULL_IDX);
    return (best_move, best_score * side_multiplier(pos.to_move));
}
*/
