use std::slice::Iter;
use std::iter::Peekable;

/*
Define block to be each 3x3 block of cells.
Idxing is done block-by-block, row-major from
top-left. E.g. index 7 is the seventh cell of
the top-left block:

0  1  2  | 9  10 11 |
3  4  5  | 12 13 14 | ...
6  7  8  | 15 16 17 |
==========================
...      |   ...    | ...
*/
pub(crate) type B33 = u16;
pub type Idx = u8;
const BOARD_SIZE: Idx = 81;
pub const NULL_IDX: Idx = 88;

macro_rules! to_local_index {
    ($index:expr) => {{
        ($index % 9) as u8
    }};
}

#[derive(Copy, Clone)]
pub struct BlockState(u8);

impl BlockState {
    // min_needed is minimum number of blocks needed for "me" to capture
    // this block. 0..3 is self-explanatory, but 4 is special: it means
    // that it is impossible for me to win this block.
    // n_routes contains the number of WIN_OCCs (i.e. rows, cols, diags)
    // that need min_needed more blocks to be won. Clearly n_routes
    // is at least 1
    pub(crate) fn new(min_needed: u8, n_routes: u8) -> BlockState {
        debug_assert!(min_needed == 4 || n_routes >= 1);
        debug_assert!(min_needed <= 4);
        // need three bits for min_needed
        BlockState(min_needed | n_routes << 3)
    }

    pub(crate) fn min_needed(&self) -> u8 {
        self.0 & 7
    }

    pub(crate) fn n_routes(&self) -> u8 {
        self.0 >> 3
    }
}

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

// 2 ^ 18
pub(crate) const N_BLOCK33: usize = 262144;
static mut BLOCK_STATE_TABLE: [BlockState; N_BLOCK33] = [BlockState(0); N_BLOCK33];
static mut INITIALIZED: bool = false;  // for sanity checks

pub fn init_moves() {
    for idx in 0..N_BLOCK33 {
        // by convention, my_occ is the lower 9 bits, etc.
        let my_occ = idx as B33 & BLOCK_OCC;
        let their_occ = (idx >> 9) as B33 & BLOCK_OCC;
        //let mut n_min: u8 = 4;
        let mut counts: [u8; 5] = [0; 5];
        let mut min_count = 4;
        for win_occ in WIN_OCC_LIST.iter() {
            if their_occ & win_occ != 0 {
                continue;  // I cannot win this route
            }
            let remaining: u8 = (3 - (win_occ & my_occ).count_ones()) as u8;
            counts[remaining as usize] += 1;
            min_count = std::cmp::min(min_count, remaining);
        }
        unsafe {
            BLOCK_STATE_TABLE[idx] = BlockState::new(min_count, counts[min_count as usize]);
        }
    }
    unsafe {
        INITIALIZED = true;
    }
}

#[inline(always)]
pub fn get_block_state(my_occ: B33, their_occ: B33) -> BlockState {
    debug_assert!(my_occ | (their_occ << 9) == my_occ + (their_occ << 9));
    get_block_state_by_idx((my_occ | (their_occ << 9)) as usize)
}

#[inline(always)]
pub fn get_block_state_by_idx(idx: usize) -> BlockState {
    unsafe {
        debug_assert!(INITIALIZED);
        BLOCK_STATE_TABLE[idx]
    }
}

// get occupancy BitVec from square
// macro_rules! sq_occ {
//     ($sq:expr) => {{
//         let sq: Square = $sq;
//         let mut bv: BitVec = empty_occ!();
//         bv.set(sq as usize, true);
//         bv
//     }};
// }

#[derive(Copy, Clone, Debug)]
pub enum GameResult {
    XWon = 0,
    OWon = 1,
    Draw = 2,
    Ongoing = 3,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Side {
    X = 0,
    O = 1,
}

impl Side {
    pub fn other(&self) -> Side {
        match self {
            Self::O => Self::X,
            Self::X => Self::O,
        }
    }

    pub fn iterator() -> Iter<'static, Side> {
        static SIDES: [Side; 2] = [Side::X, Side::O];
        return SIDES.iter();
    }
}

pub(crate) static BLOCK_OCC: B33 = 0b111111111;
pub(crate) static BLOCK_OCC_I128: i128 = BLOCK_OCC as i128;
pub(crate) static BOARD_OCC: u128 = 0x1ffffffffffffffffffffu128;

#[inline(always)]
pub(crate) fn get_block_won(occ: B33) -> bool {
    debug_assert_eq!(occ & !BLOCK_OCC, 0);
    // only need my occ
    get_block_state(occ, 0).min_needed() == 0
}

// returns true if winning is hopeless for THE OTHER SIDE
pub(crate) fn get_block_hopeless(occ: B33) -> bool {
    debug_assert_eq!(occ & !BLOCK_OCC, 0);
    // only need their occ
    get_block_state(0, occ).min_needed() == 4
}

// returns filled block occ if filled is true; 0 otherwise
#[inline(always)]
fn bool_to_block(filled: bool) -> u128 {
    ((0i128 - (filled as i128)) & BLOCK_OCC_I128) as u128
}

#[derive(Copy, Clone)]
pub struct Moves(u128);

impl Moves {
    pub fn size(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn contains(&self, index: Idx) -> bool {
        self.0 & (1u128 << index) != 0
    }

    pub fn remove(&mut self, index: Idx) {
        self.0 &= !(1u128 << index);
    }
}

impl Iterator for Moves {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0 => None,
            n => {
                let i = n.trailing_zeros();
                self.0 &= !(1 << i);
                Some(i as Idx)
            }
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct Bitboard(u128);

impl Bitboard {
    fn new() -> Bitboard {
        Bitboard(0)
    }

    // returns block index
    pub(crate) fn set(&mut self, index: Idx) -> u8 {
        debug_assert!(index < BOARD_SIZE);
        //debug_assert_eq!(self.0 & (1u128 << index), 0);
        self.0 |= 1u128 << index;

        // update block occupancy if won this block
        let block_i = index as u8 / 9;
        let block = self.get_block(block_i);
        let won = get_block_won(block);
        self.0 |= (won as u128) << (BOARD_SIZE + block_i);

        // completely fill captured block since it doesn't make
        // a difference now anyways. This also allows for easier
        // movegen & possibly later transposition
        self.0 |= bool_to_block(won) << (block_i * 9);

        return block_i;
    }

    pub fn get(&self, index: Idx) -> bool {
        debug_assert!(index < BOARD_SIZE);
        self.0 & ((1 as u128) << index) != 0
    }

    // NOTE block must be empty before this
    pub fn set_block(&mut self, block_i: u8, occ: B33) {
        debug_assert!(block_i < 9);
        debug_assert_eq!(occ & !BLOCK_OCC, 0);
        self.0 |= (occ as u128) << (block_i * 9);
        let won = get_block_won(occ);
        self.0 |= (won as u128) << (BOARD_SIZE + block_i);
        self.0 |= bool_to_block(won) << block_i * 9;
    }

    // return aligned occupancy for one block
    pub fn get_block(&self, block_i: u8) -> B33 {
        debug_assert!(block_i < 9);
        ((self.0 >> (block_i * 9)) as B33) & BLOCK_OCC
    }

    #[inline(always)]
    pub fn captured_occ(&self) -> B33 {
        ((self.0 >> BOARD_SIZE) as B33) & BLOCK_OCC
    }

    #[inline(always)]
    pub fn has_captured(&self, block_i: u8) -> bool {
        debug_assert!(block_i < 9);
        self.0 & (1 << (block_i + BOARD_SIZE)) != 0
    }

    #[inline(always)]
    pub fn n_captured(&self) -> u8 {
        return self.captured_occ().count_ones() as u8;
    }
}

#[derive(Copy, Clone)]
pub struct Position {
    pub(crate) bitboards: [Bitboard; 2],
    pub(crate) to_move: Side,
    // occupancy of blocks that cannot be won
    pub(crate) hopeless_occ: [B33; 2],
    pub(crate) last_block: u8,
}

const ANY_BLOCK: u8 = 9;

impl Position {
    pub fn new() -> Position {
        Position {
            bitboards: [Bitboard::new(), Bitboard::new()],
            to_move: Side::X,
            hopeless_occ: [0; 2],
            last_block: ANY_BLOCK,
        }
    }

    // does NOT check for termination, i.e. if the game is won/drawn
    pub fn legal_moves(&self) -> Moves {
        debug_assert!(!self.is_over());
        let total_occ = self.bitboards[0].0 | self.bitboards[1].0;
        // check if I can go anywhere on the board
        let full_board = self.last_block == ANY_BLOCK
            || (((total_occ >> (self.last_block * 9)) as B33) & BLOCK_OCC == BLOCK_OCC);
        // no-branch map full_board true => all one's, or full_board false => local one's
        let local_occ = ((BLOCK_OCC as u128) << (self.last_block * 9)) as u128;
        let mask = (0i128 - full_board as i128) as u128 | local_occ;
        Moves(mask & !total_occ & BOARD_OCC)
    }

    #[inline(always)]
    pub fn get_result(&self) -> GameResult {
        if self.is_won(Side::X) {
            return GameResult::XWon;
        } else if self.is_won(Side::O) {
            return GameResult::OWon;
        } else if self.is_drawn() {
            return GameResult::Draw;
        } else {
            return GameResult::Ongoing;
        }
    }

    #[inline(always)]
    pub fn side_to_move(&self) -> Side {
        self.to_move
    }

    #[inline(always)]
    pub fn is_won(&self, side: Side) -> bool {
        get_block_won(self.bitboards[side as usize].captured_occ())
    }

    // TODO use this for eval
    // This is test for if the game cannot be won anymore. In contrast
    // to is_drawn which only returns true for boards without any more moves.
    #[inline(always)]
    pub fn is_hopeless(&self) -> bool {
        get_block_hopeless(self.hopeless_occ[0]) && get_block_hopeless(self.hopeless_occ[1])
    }

    // NOTE does not check for win/loss. So only call this after is_won is called
    // for both sides
    #[inline(always)]
    pub fn is_drawn(&self) -> bool {
        (self.bitboards[0].0 | self.bitboards[1].0) & BOARD_OCC == BOARD_OCC
    }

    #[inline(always)]
    pub fn is_over(&self) -> bool {
        self.is_won(Side::X) || self.is_won(Side::O) || self.is_drawn()
    }

    pub fn make_move(&mut self, index: Idx) {
        // place piece
        let own_bb = &mut self.bitboards[self.to_move as usize];
        let bi = own_bb.set(index);
        let block_occ = own_bb.get_block(bi);

        // update to_move
        self.to_move = self.to_move.other();

        // update hopeless occ for the other player
        self.hopeless_occ[self.to_move as usize] |= (get_block_hopeless(block_occ) as B33) << bi;

        // update last_block
        self.last_block = to_local_index!(index);
    }

    #[allow(dead_code)]
    // returns bool so that we can put this in an assert! macro and
    // not have this code run in production
    pub fn assert(&self) -> bool {
        // bit representations are within range
        debug_assert_eq!(self.bitboards[0].0 >> (BOARD_SIZE + 9), 0);
        debug_assert_eq!(self.bitboards[1].0 >> (BOARD_SIZE + 9), 0);

        return true;
    }

    // current ply number
    pub fn cur_ply(&self) -> u16 {
        (self.bitboards[0].0 | self.bitboards[1].0).count_ones() as u16
    }
}

#[allow(dead_code)]
pub fn perft(depth: u16, pos: &mut Position) -> u64 {
    debug_assert!(pos.assert());
    if pos.is_won(pos.to_move.other()) || pos.is_drawn() {
        return 0;
    }
    if depth == 0 {
        return pos.legal_moves().size() as u64;
    }
    let mut count: u64 = 0;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        temp.make_move(mov);
        count += perft(depth - 1, &mut temp);
    }
    return count;
}

#[allow(dead_code)]
fn divide(depth: u16, pos: &mut Position) {
    debug_assert!(pos.assert());

    // special case; need to do this since perft doesn't output
    // 1 move for base case
    if depth == 0 {
        for mov in pos.legal_moves() {
            println!("{}: 1", mov);
        }
    } else {
        for mov in pos.legal_moves() {
            let mut temp = pos.clone();
            // let last_block = pos.last_block;
            temp.make_move(mov);
            let count = perft(depth - 1, &mut temp);
            // pos.unmake_move(mov, last_block);
            println!("{}: {}", mov, count);
        }
    }
}

#[allow(dead_code)]
pub fn perft_with_progress(depth: u16, pos: &mut Position) {
    debug_assert!(pos.assert());

    let moves = pos.legal_moves();
    // special case; need to do this since perft doesn't output
    // 1 move for base case
    if depth == 0 {
        println!("Done: {}", moves.size());
    } else {
        let mut total = 0;
        for mov in moves {
            let mut temp = pos.clone();
            temp.make_move(mov);
            let count = perft(depth - 1, &mut temp);
            total += count;
            println!("move {} out of {}: {}", mov + 1, moves.size(), count);
        }
        println!("Done. Total: {}", total);
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_blockwon() {
        init_moves();
        assert!(get_block_won(0b111111111));
        assert!(get_block_won(0b111000000));
        assert!(!get_block_won(0b000000000));
    }
}

