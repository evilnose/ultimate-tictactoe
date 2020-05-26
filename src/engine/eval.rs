use crate::engine::*;
use crate::moves::*;
use bitintr::Popcnt;
use std::sync::Once;

pub type EvalFn = fn(&Position) -> Score;

// enumeration of the rows, diagonals & cols
static WIN_OCC_LIST: [B33; 8] = [
    0b111,
    0b111000,
    0b111000000,
    0b001001001,
    0b010010010,
    0b100100100,
    0b001010100,
    0b100010001,
];

// 262144 = 2^18
// note that this is not optimal size (<< 3^9) but it's a hassle
// to hash it (prob slower) and there is only one such table so
// it's left as this for now.
static mut BLOCK_SCORE_TABLE: [Score; 262144] = [0.0; 262144];
static INIT_BLOCK_SCORE_TABLE: Once = Once::new();
// this maps the minimum number of cells a player needs to
// occupy to win a particular block to a score. e.g.
// if a player needs to occupy at mostone more cell to win
// this block, the evaluated score is COUNT_TO_SCORE[1].
// if the player has already won the block, the score is at 0,
// and if the player has no chance of winning the block, the
// score is at 4
//static COUNT_TO_SCORE: [i8; 5] = [8, 4, 2, 1, 0];

// Scores associated with each situation in a block
static SC_BLOCK_WON: Score = 8.0;
static SC_NEED_1: Score = 4.0;
static SC_NEED_2: Score = 0.1;
static SC_NEED_3: Score = 0.1;
static SC_HOPELESS: Score = 0.0;  // no chance of winning this block
/*
static SC_NEED_2_2: Score = 100;
static SC_NEED_1_3P: Score = 100.0;
static SC_NEED_1_2: Score = 100.0;
*/

// return no. of cells to need to occupy to win. 4 if
// I can't win
fn eval_block_side(my_occ: B33, their_occ: B33) -> Score {
    //let mut n_min: u8 = 4;
    let mut counts: [u8; 4] = [0; 4];
    for win_occ in WIN_OCC_LIST.iter() {
        if their_occ & win_occ != 0 {
            continue; // no way I can win this
        }
        let remaining: u8 = (3 - (win_occ & my_occ).popcnt()) as u8;
        //n_min = std::cmp::min(n_min, remaining);
        counts[remaining as usize] += 1;
    }
    if counts[0] != 0 {
        SC_BLOCK_WON
    } else if counts[1] != 0 {
        SC_NEED_1 * (counts[1] as Score).sqrt()
    } else if counts[2] != 0 {
        SC_NEED_2 * (counts[2] as Score).sqrt()
    } else if counts[3] != 0 {
        SC_NEED_3
    } else {
        SC_HOPELESS
    }
}

// evaluate a 3x3 block, given the occupancy of the two players
// the more positive (less negative) the better for X
#[inline]
fn eval_block(x_occ: B33, o_occ: B33) -> Score {
    debug_assert!(x_occ | o_occ << 9 == x_occ + o_occ << 9);
    unsafe {
        INIT_BLOCK_SCORE_TABLE.call_once(|| {
            for idx in 0..262144 {
                let xo = idx as B33 & BLOCK_OCC;
                let oo = (idx >> 9) as B33 & BLOCK_OCC;
                BLOCK_SCORE_TABLE[idx] = eval_block_side(xo, oo) - eval_block_side(oo, xo);
            }
        });
        BLOCK_SCORE_TABLE[(x_occ | o_occ << 9) as usize]
    }
}

/*
// return the score for one side. After this is called, the score
// should be negated for O by convention
#[inline]
pub fn side_score(pos: &Position, side: Side) -> Score {
    let bb = &pos.bitboards[side as usize];
    let mut base_score = bb.captured_occ().count_ones() as Score;
    for bi in 0..9 {
        base_score += 0.05 * (bb.get(bi * 9 + 4) as i32 as f32);
    }
    // additional score for center block captured
    return base_score + 0.5 * bb.has_captured(4) as i32 as f32;
}
*/

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
    let big_score = eval_block(pos.bitboards[0].captured_occ(), pos.bitboards[1].captured_occ());
    ret += big_score * 100.0;
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

// simple helper function that returns 1 if equal
// is true and -1 if not
#[inline(always)]
pub(crate) fn side_multiplier(side: Side) -> Score {
    (1 - 2 * (side as i32)) as Score
}
