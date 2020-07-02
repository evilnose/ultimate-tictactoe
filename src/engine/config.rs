
pub type Score = f32;

// don't exceed these pls
pub(crate) const SCORE_NEG_INF: f32 = -1000000.0;
pub(crate) const SCORE_POS_INF: f32 = 1000000.0;

/* SEARCH PARAMETERS */

// add noise to eval if the remaining search depth equals to this
pub(crate) const NOISE_DEPTH: u16 = 5;

// add noise if the number of moves made is le
// NOISE_MOVE_MAX and ge NOISE_MOVES_MIN
//pub(crate) const NOISE_MOVES_MAX: usize = 0;
//pub(crate) const NOISE_MOVES_MIN: usize = 0;
