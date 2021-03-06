use crate::engine::config::*;
use crate::engine::utils::*;
use crate::moves::*;

pub type EvalFn = fn(&Position) -> Score;

// 262144 = 2^18
// note that this is not optimal size (<< 3^9) but it's a hassle
// to hash it (prob slower) and there is only one such table so
// it's left as this for now.
// NOTE: for now this is not used. though if necessary, this
// can be part of the initialization, i.e. instead of
// blockstate_to_score[occ_to_blockstate[occ]] simply do
// block_score_table[occ]. takes a bit more memory though.
//static mut BLOCK_SCORE_TABLE: [Score; 262144] = [0.0; 262144];

// Scores associated with each situation in a block
/*
static SC_BLOCK_WON: Score = 4.0;
static SC_NEED_1: Score = 3.0;
static SC_NEED_2: Score = 1.5; // TODO this can't be the same as 3
static SC_NEED_3: Score = 1.0;
static SC_HOPELESS: Score = 0.0; // no chance of winning this block
*/
static SC_BLOCK_WON: Score = 8.0;
static SC_NEED_1: Score = 3.0;
static SC_NEED_2: Score = 0.5; // TODO this can't be the same as 3
static SC_NEED_3: Score = 0.1;
static SC_HOPELESS: Score = 0.0; // no chance of winning this block
static BIG_SCORE_MULT: Score = 10.0;
                                 // some arbitrariliy decided sublinear function for 0-9; values capped at 2
static SUBLINEAR_5: [Score; 10] = [1.0, 1.4, 1.7, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0];

static mut BLOCK_SCORE_TABLE: [f32; N_BLOCK33] = [0.0; N_BLOCK33];
static mut DOUBLE_MAX_SCORE: Score = 0.0;

#[inline(always)]
pub fn get_double_max_score() -> Score {
    unsafe {
        return DOUBLE_MAX_SCORE;
    }
}

/*
pub fn init_block_score_table() {
    unsafe {
        // assuming SC_HOPELESS is 0
        DOUBLE_MAX_SCORE = SC_BLOCK_WON * BIG_SCORE_MULT + 9.0 * SC_BLOCK_WON * 2.0;
        for idx in 0..N_BLOCK33 {
            let bs = get_block_state_by_idx(idx);
            /*
            BLOCK_SCORE_TABLE[idx] = match bs.min_needed() {
                0 => SC_BLOCK_WON,
                1 => SC_NEED_1 * SUBLINEAR_5[bs.n_routes() as usize],
                2 => SC_NEED_2 * bs.n_routes() as Score,
                3 => SC_NEED_3,
                4 => SC_HOPELESS,
                _ => panic!("min_needed is not in range [0, 4]"),
            };
            */
            BLOCK_SCORE_TABLE[idx] = match bs.min_needed() {
                0 => SC_BLOCK_WON,
                1 => SC_NEED_1,
                2 => SC_NEED_2,
                3 => SC_NEED_3,
                4 => SC_HOPELESS,
                _ => panic!("min_needed is not in range [0, 4]"),
            };
        }
    }
}
*/
pub fn init_block_score_table() {
    unsafe {
        DOUBLE_MAX_SCORE = SC_BLOCK_WON * BIG_SCORE_MULT + 9.0 * SC_BLOCK_WON * 2.0;
        for idx in 0..N_BLOCK33 {
            let bs = get_block_state_by_idx(idx);
            /*
            BLOCK_SCORE_TABLE[idx] = match bs.min_needed() {
                0 => SC_BLOCK_WON,
                1 => SC_NEED_1 * SUBLINEAR_5[bs.n_routes() as usize],
                2 => SC_NEED_2 * bs.n_routes() as Score,
                3 => SC_NEED_3,
                4 => SC_HOPELESS,
                _ => panic!("min_needed is not in range [0, 4]"),
            };
            */
            BLOCK_SCORE_TABLE[idx] = match bs.min_needed() {
                0 => SC_BLOCK_WON,
                1 => SC_NEED_1 * SUBLINEAR_5[bs.n_routes() as usize],
                2 => SC_NEED_2 * SUBLINEAR_5[bs.n_routes() as usize],
                3 => SC_NEED_3,
                4 => SC_HOPELESS,
                _ => panic!("min_needed is not in range [0, 4]"),
            };
        }
    }
}

// evaluate a 3x3 block, given the occupancy of the two players
// the more positive (less negative) the better for X
#[inline(always)]
pub fn eval_block(x_occ: B33, o_occ: B33) -> Score {
    // TODO zero out opponent's block when you capture a whole block. I am NOT already doing that
    // since the below assert will fail if I leave it with only the last expression
    debug_assert!(get_block_won(o_occ) || get_block_won(x_occ) || ((x_occ | o_occ) == (x_occ + o_occ)));
    unsafe {
        BLOCK_SCORE_TABLE[(x_occ | (o_occ << 9)) as usize]
            - BLOCK_SCORE_TABLE[(o_occ | (x_occ << 9)) as usize]
    }
}

// NOTE: called when pos is not won/lost/drawn; may not work
// correctly otherwise, and no checks are performed
pub fn eval(pos: &Position) -> Score {
    let side2move = side_multiplier(pos.to_move);
    let mut ret: Score = 0.0;
    for bi in 0..9 {
        ret += eval_block(
            pos.bitboards[0].get_block(bi),
            pos.bitboards[1].get_block(bi),
        );
    }
    let big_score = eval_block(
        pos.bitboards[0].captured_occ(),
        pos.bitboards[1].captured_occ(),
    );
    ret += big_score * BIG_SCORE_MULT;

    return ret * side2move;
    /*
    // only need to check if the side just moved has won
    return (side_score(&pos, Side::X) - side_score(&pos, Side::O)) * side2move;
    */
}

pub fn basic_eval(pos: &Position) -> Score {
    let side2move = side_multiplier(pos.to_move);
    return (pos.bitboards[0].captured_occ().count_ones() as Score
        - pos.bitboards[1].captured_occ().count_ones() as Score)
        * side2move;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_eval() {
        let pos = Position::new();
        basic_eval(&pos);
    }
}
