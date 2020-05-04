use crate::moves::*;
use crate::engine::*;

pub fn basic_eval(pos: &Position) -> Score {
    let side2move: Score = -1 + 2 * (pos.to_move == Side::X) as Score;
    match pos.get_result() {
        GameResult::X_WON => 100,
        GameResult::O_WON => -100,
        GameResult::DRAW => 0,
        GameResult::ONGOING => {
            (pos.bitboards[0].captured_occ().count_ones() as Score -
            pos.bitboards[1].captured_occ().count_ones() as Score) * side2move
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
