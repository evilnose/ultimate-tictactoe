use crate::engine::*;
use crate::moves::*;

pub type EvalFn = fn(&Position) -> Score;

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

// called when pos is not won/lost/drawn
pub fn eval(pos: &Position) -> Score {
    let side2move = side_multiplier(pos.to_move);
    // only need to check if the side just moved has won
    return (side_score(&pos, Side::X) - side_score(&pos, Side::O)) * side2move;
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
