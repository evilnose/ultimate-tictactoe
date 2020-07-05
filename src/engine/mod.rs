pub mod config;
pub mod eval;
pub mod utils;

use std::time::Instant;
use std::thread;
use std::sync::mpsc;
use std::fmt;

use crate::engine::config::*;
use crate::engine::eval::*;
use crate::moves::*;

// used to signal that search has stopped
// TODO improve this
#[derive(Debug)]
struct StopSearch;

// sent to the manager to update state of the search periodically,
// probably every time the depth increases
enum SearchInfo {
    Update {
        best_move: Idx,
        eval: Score
    },
    Terminate,
}

// keeps track of the current search state, e.g. node count, time,
// etc. Used across alpha_beta_dfs
struct SearchState {
    start_search: Instant,
    alloc_millis: u64,
    nodes_searched: u64,
}

impl SearchState {
    fn new(alloc_millis : u64) -> SearchState {
        SearchState {
            start_search: Instant::now(),
            alloc_millis: alloc_millis,
            nodes_searched: 0,
        }
    }
}

pub struct SearchResult {
    pub best_move: Idx,
    pub eval: Score,
}

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

    pub fn search_fixed_time(&self, alloc_millis: u64) -> SearchResult {
        let (tx, rx) = mpsc::channel();
        let localpos = self.position;
        
        thread::spawn(move || {
            let mut worker = Worker::new(localpos, tx);
            worker.search_fixed_time(alloc_millis);
        });

        let mut latest = SearchResult {
            best_move: NULL_IDX,
            eval: 0.0,
        };

        loop {
            match rx.try_recv() {
                Ok(info) => {
                    match info {
                        SearchInfo::Update {
                            best_move: b,
                            eval: e
                        } => {
                            latest.best_move = b;
                            latest.eval = e;
                        },
                        SearchInfo::Terminate => {
                            assert!(latest.best_move != NULL_IDX);
                            return latest;
                        },
                    }
                },
                Err(mpsc::TryRecvError::Empty) => {},
                Err(mpsc::TryRecvError::Disconnected) => panic!("ERROR: search channel disconnected"),
            }
        }
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
    sender: mpsc::Sender<SearchInfo>,
}

impl Worker {
    pub fn new(pos: Position, tx: mpsc::Sender<SearchInfo>) -> Worker {
        Worker {
            position: pos,
            eval_fn: eval, // default to eval; might change later
            sender: tx,
        }
    }

    // TODO implement this with threads
    fn search_till_depth(&mut self, depth: u16) -> Score {
        let mut state = SearchState::new(std::u64::MAX);
        // no need for clone since self.position is implicitly copied
        self.alpha_beta_dfs(depth, self.position, SCORE_NEG_INF, SCORE_POS_INF, &mut state).unwrap()
    }

    // TODO implement this in manager
    pub fn search_fixed_time(&mut self, alloc_millis: u64) {
        let mut state = SearchState::new(alloc_millis);

        let moves = self.position.legal_moves();

        let mut t_moves = moves.clone();
        let mut best = t_moves.next().expect("error: no legal moves");
        let mut best_score = SCORE_NEG_INF;

        for depth in 1..=MAX_SEARCH_PLIES {
            let mut localmoves = moves.clone();

            // search last best move first
            {
                let mut localpos = self.position.clone();
                localpos.make_move(best);
                let res = self.alpha_beta_dfs(depth - 1, localpos, SCORE_NEG_INF, SCORE_POS_INF, &mut state);
                best_score = match res {
                    Ok(s) => -s,
                    Err(_e) => return,
                };
                localmoves.remove(best);
            }

            // search the remaining moves
            for mv in localmoves {
                self.sender.send(SearchInfo::Update{
                    best_move: best,
                    eval: best_score,
                }).unwrap();
                let mut localpos = self.position.clone();
                localpos.make_move(mv);
                // note that best_score acts as alpha here TODO is this right?
                let res = self.alpha_beta_dfs(depth - 1, localpos, SCORE_NEG_INF, -best_score, &mut state);
                let score = match res {
                    Ok(s) => -s,
                    Err(_e) => return,
                };
                if score > best_score {
                    best_score = score;
                    best = mv;
                }
            }
            eprintln!("NOTE: depth {}/move {}/best {}/", depth, best, best_score);
        }

        eprintln!("NOTE: stopping search since MAX_SEARCH_PLIES exceeded");
        self.sender.send(SearchInfo::Update{
            best_move: best,
            eval: best_score,
        }).unwrap();
        self.sender.send(SearchInfo::Terminate).unwrap();
    }

    /*
    // alpha-beta iterative deepening search
    fn limited_search(&self, pos: &Position, alpha: Score, beta: Score) -> SearchResult {
        debug_assert!(pos.assert());
    }
    */

    // alpha-beta negamax search using DFS
    // TODO return SearchResult instead
    fn alpha_beta_dfs(&mut self, depth: u16, pos: Position, alpha: Score, beta: Score, state: &mut SearchState) -> Result<Score, StopSearch> {
        debug_assert!(pos.assert());

        // this move has won -- it's terrible for the current side
        // note only the last moved side could have won so only
        // one call to is_won() is made
        // TODO do we need to check for this? would alpha-beta take care of this
        if pos.is_won(pos.to_move.other()) {
            state.nodes_searched += 1;
            return self.check_time(SCORE_LOSS, state);
        } else if pos.is_drawn() {
            state.nodes_searched += 1;
            // NOTE that one side could still be considered won in some rulesets by comparing
            // the total number of blocks occupied
            let diff = pos.bitboards[0].n_captured() as i16 - pos.bitboards[1].n_captured() as i16;
            if diff != 0 {
                let sign = (diff as f32).signum();
                // e.g. if sign is positive and side is X then it's very good.
                return self.check_time(sign * side_multiplier(pos.to_move) * SCORE_WIN, state);
            } else {
                // actually dead drawn
                return self.check_time(0.0, state);
            }
        } else if depth == 0 {
            state.nodes_searched += 1;
            let f = self.eval_fn;
            return self.check_time(f(&pos), state);
        }

        let mut alpha = alpha;

        for mov in pos.legal_moves() {
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
        return Ok(alpha);
    }

    // called when a leaf node is reached. If timed out, return Err(StopSearch). Otherwise
    // return the given eval wrapped in Result
    #[inline(always)]
    fn check_time(&mut self, eval: Score, state: &SearchState) -> Result<Score, StopSearch> {
        if state.nodes_searched % 1000 == 0 {
            if (state.alloc_millis as i64 - state.start_search.elapsed().as_millis() as i64) < 5 {
                self.sender.send(SearchInfo::Terminate).unwrap();
                return Err(StopSearch);
            }
        }

        return Ok(eval);
    }
}

pub fn init_engine() {
    init_block_score_table();
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
