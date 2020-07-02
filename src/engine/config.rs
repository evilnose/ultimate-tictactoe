use crate::moves::*;

pub type Score = f32;

// don't exceed these pls
pub(crate) const SCORE_LOSS: f32 = -1e6;
pub(crate) const SCORE_WIN: f32 = 1e6;
pub(crate) const SCORE_NEG_INF: f32 = -1e7;
pub(crate) const SCORE_POS_INF: f32 = 1e7;

pub struct SearchResult {
    pub best_move: Idx,
    pub eval: Score,
}

pub(crate) const MAX_SEARCH_PLIES: u16 = 20;
