use crate::moves::*;
use crate::engine::*;

pub fn basic_eval(pos: &Position) -> Score {
    let side2move: Score = -1 + 2 * (pos.to_move == Side::X) as Score;
    match pos.get_result() {
        GameResult::XWon => 100 * side2move,
        GameResult::OWon => -100 * side2move,
        GameResult::Draw => 0,
        GameResult::Ongoing => {
            (pos.bitboards[0].captured_occ().count_ones() as Score -
            pos.bitboards[1].captured_occ().count_ones() as Score) * side2move
        }
    }
}

// return very large, very negative, or zero if game is over.
// otherwise return -1. This encapsulates side2move.
pub fn end_check(pos: &Position) -> Score {
    let side2move: Score = -1 + 2 * (pos.to_move == Side::X) as Score;
    match pos.get_result() {
        GameResult::XWon => 100 * side2move,
        GameResult::OWon => -100 * side2move,
        GameResult::Draw => 0,
        GameResult::Ongoing => {
            -1
        }
    }
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
