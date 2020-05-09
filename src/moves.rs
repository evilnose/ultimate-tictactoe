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
pub type Idx = u16;
const BOARD_SIZE: Idx = 81;
pub const NULL_IDX: Idx = 81;

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

pub(crate) fn block_won(occ: B33) -> bool {
    debug_assert_eq!(occ & !BLOCK_OCC, 0);
    WIN_TABLE[occ as usize / 64] & (1 << (occ % 64)) != 0
}

// returns true if winning is hopeless for THE OTHER SIDE
pub(crate) fn block_hopeless(occ: B33) -> bool {
    debug_assert_eq!(occ & !BLOCK_OCC, 0);
    HOPELESS_TABLE[occ as usize / 64] & (1 << (occ % 64)) != 0
}

#[derive(Copy, Clone)]
pub struct Moves {
    occupancy: [u64; 2],
}

impl Moves {
    fn new() -> Moves {
        Moves { occupancy: [0; 2] }
    }

    fn add(&mut self, index: Idx) {
        self.occupancy[index as usize / 63] |= 1u64 << (index % 63);
    }

    pub fn size(&self) -> u32 {
        self.occupancy[0].count_ones() + self.occupancy[1].count_ones()
    }

    pub fn contains(&self, index: Idx) -> bool {
        self.occupancy[index as usize / 63] & (1u64 << (index % 63)) != 0
    }
}

impl Iterator for Moves {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = (self.occupancy[0] == 0) as usize;
        let occ = self.occupancy[idx];

        if occ == 0 {
            return None;
        } else {
            let i = occ.tzcnt() as Idx;
            self.occupancy[idx] &= !(1 << i);
            return Some(idx as Idx * 63 + i);
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct Bitboard {
    // occupancy[0]: first 63 bits: first 7 blocks;
    // occupancy[1]: first 18 bits: last 2 blocks;
    //               next 9 bits: block occ
    occupancy: [u64; 2],
}

impl Bitboard {
    fn new() -> Bitboard {
        Bitboard { occupancy: [0; 2] }
    }

    // returns block index
    pub(crate) fn set(&mut self, index: Idx) -> u8 {
        debug_assert!(index < BOARD_SIZE);
        debug_assert_eq!(
            self.occupancy[index as usize / 63] & (1u64 << (index % 63)),
            0
        );
        self.occupancy[index as usize / 63] |= 1u64 << (index % 63);

        // update block occupancy if won this block
        let block_i = index as u8 / 9;
        let block = self.get_block(block_i);
        self.occupancy[1] |= (block_won(block) as u64) << (18 + block_i);

        return block_i;
    }

    // returns large hopeless block occ if a block becomes hopeful again
    pub(crate) fn unset(&mut self, index: Idx) -> u8 {
        debug_assert!(index < BOARD_SIZE);
        assert_ne!(
            self.occupancy[index as usize / 63] & (1u64 << (index % 63)),
            0
        );
        self.occupancy[index as usize / 63] &= !(1u64 << (index % 63));

        // update block occupancy if un-won this block
        let block_i = index as u8 / 9;
        let block = self.get_block(block_i);
        self.occupancy[1] &= !((block_won(block) as u64) << (18 + block_i));
        // return (block_hopeless(block) as B33) << block_i;
        return block_i;
    }

    // NOTE block must be empty before this
    pub fn set_block(&mut self, block_i: u8, occ: B33) {
        debug_assert_eq!(occ & !BLOCK_OCC, 0);
        // set
        self.occupancy[block_i as usize / 7] |= (occ as u64) << ((block_i % 7) * 9);
        // set
        self.occupancy[1] |= (block_won(occ) as u64) << (18 + block_i);
    }

    // return aligned occupancy for one block
    pub fn get_block(&self, block_i: u8) -> B33 {
        ((self.occupancy[block_i as usize / 7] >> ((block_i % 7) * 9)) as B33) & BLOCK_OCC
    }

    pub fn get(&self, index: Idx) -> bool {
        if index >= BOARD_SIZE {
            panic!("Bitboard::get() out of bounds")
        }
        (self.occupancy[index as usize / 63] & ((1 as u64) << (index % 63))) != 0
    }

    pub fn captured_occ(&self) -> B33 {
        ((self.occupancy[1] >> 18) as B33) & BLOCK_OCC
    }

    // // returns precomputed result: if 3x3 block is captured/drawn
    // pub fn block_marked(&self, block_i: u8) -> bool {
    //     (self.occupancy[1] | (1 << (18 + block_i))) != 0
    // }
}

/*
Need 81 * 2 bits for each player's general board
4 bits for the last large square played
1 bit for the side to move

167 bits in total. 3 longs.
*/
#[derive(Copy, Clone)]
pub struct Position {
    pub(crate) bitboards: [Bitboard; 2],
    pub(crate) to_move: Side,
    pub(crate) full_blocks: B33, // occupancy of blocks that are full
    pub(crate) hopeless_occ: [B33; 2], // occupancy of blocks that cannot be won
    pub(crate) last_block: u8,
}

const ANY_BLOCK: u8 = 9;

impl Position {
    pub fn new() -> Position {
        Position {
            bitboards: [Bitboard::new(), Bitboard::new()],
            to_move: Side::X,
            full_blocks: 0,
            hopeless_occ: [0; 2],
            last_block: ANY_BLOCK,
        }
    }

    pub fn legal_moves(&self) -> Moves {
        if self.is_over() {
            return Moves::new();
        }
        // TODO check for inline performance
        return self.legal_moves_nocheck();
    }

    // does NOT check for termination, i.e. if the game is won/drawn
    pub fn legal_moves_nocheck(&self) -> Moves {
        let mut moves = Moves::new();
        let dead_blocks =
            self.bitboards[0].captured_occ() | self.bitboards[1].captured_occ() | self.full_blocks;
        if self.last_block == ANY_BLOCK || dead_blocks & (1 << self.last_block) != 0 {
            // can go anywhere that is not captured
            let mut blocks: B33 = !dead_blocks & BLOCK_OCC;
            while blocks != 0 {
                let block_i = blocks.tzcnt();
                self.add_block_moves(block_i as u8, &mut moves);
                blocks &= !(1 << block_i);
            }
        } else {
            self.add_block_moves(self.last_block, &mut moves);
        }

        return moves;
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
        block_won(self.bitboards[side as usize].captured_occ())
    }

    // TODO use this for eval
    // This is test for if the game cannot be won anymore. In contrast
    // to is_drawn which only returns true for boards without any more moves.
    #[inline(always)]
    pub fn is_hopeless(&self) -> bool {
        panic!("is_hopeless() not implemented. You probably want is_draw() for now");
        // block_hopeless(self.hopeless_occ[0]) && block_hopeless(self.hopeless_occ[1])
    }

    #[inline(always)]
    pub fn is_drawn(&self) -> bool {
        self.full_blocks | self.bitboards[0].captured_occ() | self.bitboards[1].captured_occ()
            == BLOCK_OCC
    }

    #[inline(always)]
    pub fn is_over(&self) -> bool {
        self.is_won(Side::X) || self.is_won(Side::O) || self.is_drawn()
    }

    fn add_block_moves(&self, block_i: u8, moves: &mut Moves) {
        let mut block_occ = !self.both_block_occ(block_i) & BLOCK_OCC;
        let offset = block_i as Idx * 9;
        while block_occ != 0 {
            let idx = block_occ.tzcnt();
            moves.add(idx + offset);
            block_occ &= !(1 << idx);
        }
    }

    #[inline(always)]
    fn both_block_occ(&self, block_i: u8) -> B33 {
        self.bitboards[0].get_block(block_i) | self.bitboards[1].get_block(block_i)
    }

    pub fn make_move(&mut self, index: Idx) {
        // place piece
        let own_bb = &mut self.bitboards[self.to_move as usize];
        let bi = own_bb.set(index);
        let block_occ = own_bb.get_block(bi);

        // update full block
        self.full_blocks |= (self.is_block_full(bi) as B33) << bi;

        // update to_move
        self.to_move = self.to_move.other();

        // update hopeless occ for the other player
        self.hopeless_occ[self.to_move as usize] |= (block_hopeless(block_occ) as B33) << bi;

        // update last_block
        self.last_block = to_local_index!(index);
    }

    pub fn unmake_move(&mut self, index: Idx, last_block: u8) {
        self.to_move = self.to_move.other();

        let own_bb = &mut self.bitboards[self.to_move as usize];
        let bi = own_bb.unset(index);
        let block_occ = own_bb.get_block(bi);

        // update full block
        self.full_blocks &= !(1 << bi);

        // update hopeless occ
        self.hopeless_occ[self.to_move.other() as usize] &=
            !((!block_hopeless(block_occ) as B33) << bi);

        // update last_block
        self.last_block = last_block;
    }

    #[inline(always)]
    pub(crate) fn is_block_full(&self, block_i: u8) -> bool {
        self.both_block_occ(block_i) == BLOCK_OCC
    }

    #[allow(dead_code)]
    // returns bool so that we can put this in an assert! macro and
    // not have this code run in production
    pub fn assert(&self) -> bool {
        const B_18: u64 = 0x3FFFF;
        const B_27: u64 = 0x7FFFFFF;
        let x_occ0 = self.bitboards[Side::X as usize].occupancy[0];
        let x_occ1 = self.bitboards[Side::X as usize].occupancy[1];
        let x_big_occ = x_occ1 >> 18;
        let o_occ0 = self.bitboards[Side::O as usize].occupancy[0];
        let o_occ1 = self.bitboards[Side::O as usize].occupancy[1];
        let o_big_occ = o_occ1 >> 18;

        // occupancies don't overlap
        debug_assert_eq!(x_occ0 & o_occ0, 0);
        debug_assert_eq!((x_occ1 & o_occ1) & B_18, 0);

        // big occupancies don't overlap
        debug_assert_eq!(x_big_occ & o_big_occ, 0);

        // bit representations are within range
        debug_assert_eq!(x_occ0 & (1 << 63), 0);
        debug_assert_eq!(o_occ0 & (1 << 63), 0);
        debug_assert_eq!(x_occ1 & !B_27, 0);
        debug_assert_eq!(o_occ1 & !B_27, 0);

        return true;
    }
}

#[allow(dead_code)]
pub fn perft(depth: u16, pos: &mut Position) -> u64 {
    debug_assert!(pos.assert());
    if depth == 0 {
        return pos.legal_moves().size() as u64;
    }
    let mut count: u64 = 0;

    for mov in pos.legal_moves() {
        let mut temp = pos.clone();
        // let last_block = pos.last_block;
        temp.make_move(mov);
        count += perft(depth - 1, &mut temp);
        // pos.unmake_move(mov, last_block);
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
