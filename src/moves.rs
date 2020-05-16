use bitintr::*;
use std::slice::Iter;
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

static WIN_TABLE: [u64; 8] = [
    0xff80808080808080,
    0xfff0aa80faf0aa80,
    0xffcc8080cccc8080,
    0xfffcaa80fefcaa80,
    0xfffaf0f0aaaa8080,
    0xfffafaf0fafaaa80,
    0xfffef0f0eeee8080,
    0xffffffffffffffff,
];

// TODO
static HOPELESS_TABLE: [u64; 8] = [0; 8];

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

pub(crate) fn compute_block_won(occ: B33) -> bool {
    debug_assert_eq!(occ & !BLOCK_OCC, 0);
    WIN_TABLE[occ as usize / 64] & (1 << (occ % 64)) != 0
}

// returns true if winning is hopeless for THE OTHER SIDE
pub(crate) fn compute_block_hopeless(occ: B33) -> bool {
    debug_assert_eq!(occ & !BLOCK_OCC, 0);
    HOPELESS_TABLE[occ as usize / 64] & (1 << (occ % 64)) != 0
}

// returns filled block occ if filled is true; 0 otherwise
#[inline(always)]
fn bool_to_block(filled: bool) -> u128 {
    ((0i128 - (filled as i128)) & BLOCK_OCC_I128) as u128
}

// tzcnt() is not implemented for u128. I emulate it here
trait MyTzcnt {
    fn tzcnt(&self) -> Self;
}

impl MyTzcnt for u128 {
    fn tzcnt(&self) -> Self {
        let cnt1 = (0i64 - ((*self as u64) == 0u64) as i64) & ((*self >> 64) as u64).tzcnt() as i64;
        return cnt1 as u128 + (*self as u64).tzcnt() as u128;
    }
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
}

impl Iterator for Moves {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0 => None,
            n => {
                let i = n.tzcnt();
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
        debug_assert_eq!(self.0 & (1u128 << index), 0);
        self.0 |= 1u128 << index;

        // update block occupancy if won this block
        let block_i = index as u8 / 9;
        let block = self.get_block(block_i);
        let won = compute_block_won(block);
        self.0 |= (won as u128) << (BOARD_SIZE + block_i);

        // completely fill captured block since it doesn't make
        // a difference now anyways. This also allows for easier
        // movegen & possibly later transposition
        self.0 |= bool_to_block(won) << block_i * 9;

        return block_i;
    }

    // NOTE block must be empty before this
    pub fn set_block(&mut self, block_i: u8, occ: B33) {
        debug_assert!(block_i < 9);
        debug_assert_eq!(occ & !BLOCK_OCC, 0);
        self.0 |= (occ as u128) << (block_i * 9);
        let won = compute_block_won(occ);
        self.0 |= (won as u128) << (BOARD_SIZE + block_i);
        self.0 |= bool_to_block(won) << block_i * 9;
    }

    // return aligned occupancy for one block
    pub fn get_block(&self, block_i: u8) -> B33 {
        debug_assert!(block_i < 9);
        ((self.0 >> (block_i * 9)) as B33) & BLOCK_OCC
    }

    pub fn get(&self, index: Idx) -> bool {
        debug_assert!(index < BOARD_SIZE);
        self.0 & ((1 as u128) << index) != 0
    }

    #[inline]
    pub fn captured_occ(&self) -> B33 {
        ((self.0 >> BOARD_SIZE) as B33) & BLOCK_OCC
    }

    #[inline]
    pub fn has_captured(&self, block_i: u8) -> bool {
        debug_assert!(block_i < 9);
        self.0 & (1 << (block_i + BOARD_SIZE)) != 0
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
            || (((total_occ >> (self.last_block * 9)) as B33) & BLOCK_OCC
                == BLOCK_OCC);
        
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
        compute_block_won(self.bitboards[side as usize].captured_occ())
    }

    // TODO use this for eval
    // This is test for if the game cannot be won anymore. In contrast
    // to is_drawn which only returns true for boards without any more moves.
    #[inline(always)]
    pub fn is_hopeless(&self) -> bool {
        panic!("is_hopeless() not implemented. You probably want is_draw() for now");
        // block_hopeless(self.hopeless_occ[0]) && block_hopeless(self.hopeless_occ[1])
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
        self.hopeless_occ[self.to_move as usize] |=
            (compute_block_hopeless(block_occ) as B33) << bi;

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
