use bitintr::*;
use std::marker::PhantomData;

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
type B33 = u16;
type Idx = u16;
const BOARD_SIZE: Idx = 81;

// convert from row-major indexing to bitboard indexing
macro_rules! to_bb_index {
    ($row:expr, $col:expr) => {{
        let row: usize = $row;
        let col: usize = $col;
        let bi = ((row / 3) * 3 + (col / 3));
        let small_row = row % 3;
        let small_col = col % 3;
        (bi * 9 + (small_row * 3 + small_col)) as u16
    }};
}

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
static DRAW_TABLE: [u64; 8] = [0; 8];

// get occupancy BitVec from square
// macro_rules! sq_occ {
//     ($sq:expr) => {{
//         let sq: Square = $sq;
//         let mut bv: BitVec = empty_occ!();
//         bv.set(sq as usize, true);
//         bv
//     }};
// }

#[derive(Copy, Clone)]
pub enum Side {
    X = 0,
    O = 1,
}

impl Side {
    fn other(&self) -> Side {
        match self {
            Self::O => Self::X,
            Self::X => Self::O,
        }
    }
}

trait BlockClassifier {
    // classify block as
    fn classify_block(block: B33) -> bool;
}

#[derive(Copy, Clone)]
struct Capturer;
impl BlockClassifier for Capturer {
    fn classify_block(block: B33) -> bool {
        WIN_TABLE[block as usize / 64] & (1 << (block % 64)) != 0
    }
}

#[derive(Copy, Clone)]
struct Drawer;
impl BlockClassifier for Drawer {
    fn classify_block(block: B33) -> bool {
        DRAW_TABLE[block as usize / 64] & (1 << (block % 64)) != 0
    }
}

#[derive(Copy, Clone)]
pub struct Moves {
    occupancy: [u64; 2],
}

impl Moves {
    fn new() -> Moves {
        Moves { occupancy: [0; 2] }
    }

    fn from_bitboard(occ0: u64, occ1: u64) -> Moves {
        let mut moves = Moves::new();
        moves.occupancy[0] = occ0;
        moves.occupancy[1] = occ1;
        return moves;
    }

    fn add(&mut self, index: Idx) {
        self.occupancy[index as usize / 63] |= 1u64 << (index % 63);
    }

    fn contains(&self, index: Idx) -> bool {
        self.occupancy[index as usize / 63] & 1u64 << (index % 63) != 0
    }

    pub fn size(&self) -> u64 {
        return (self.occupancy[0].count_ones() + self.occupancy[1].count_ones()) as u64;
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
struct Bitboard<C: BlockClassifier> {
    // occupancy[0]: first 63 bits: first 7 blocks;
    // occupancy[1]: first 18 bits: last 2 blocks;
    //               next 9 bits: block occ
    occupancy: [u64; 2],
    phantom: PhantomData<C>,
}

static BLOCK_OCC: B33 = 0b111111111;

impl<C: BlockClassifier> Bitboard<C> {
    pub fn new() -> Bitboard<C>
    where
        C: BlockClassifier,
    {
        Bitboard::<C> {
            occupancy: [0; 2],
            phantom: PhantomData,
        }
    }

    // returns captured block occupancy if a block is captured
    pub fn set(&mut self, index: Idx) -> B33 {
        assert!(index < BOARD_SIZE);
        assert_eq!(
            self.occupancy[index as usize / 63] & (1u64 << (index % 63)),
            0
        );
        self.occupancy[index as usize / 63] |= 1u64 << (index % 63);

        // update block occupancy if won this block
        let block_i = index as u8 / 9;
        let block = self.block_occ(block_i);

        let block_marked = C::classify_block(block) as u64;
        self.occupancy[1] |= block_marked << (18 + block_i);
        return (block_marked as B33) << block_i;
    }

    pub fn unset(&mut self, index: Idx) -> B33 {
        assert!(index < BOARD_SIZE);
        assert_ne!(
            self.occupancy[index as usize / 63] & (1u64 << (index % 63)),
            0
        );
        self.occupancy[index as usize / 63] &= !(1u64 << (index % 63));

        // update block occupancy if un-captured this block
        let block_i = index as u8 / 9;
        let block = self.block_occ(block_i);

        if !C::classify_block(block) {
            self.occupancy[1] &= !(1 << (18 + block_i));
            return 1 << block_i;
        }
        return 0;
    }

    fn block_occ(&self, block_i: u8) -> B33 {
        ((self.occupancy[block_i as usize / 7] >> ((block_i % 7) * 9)) as B33) & BLOCK_OCC
    }

    pub fn get(&self, index: Idx) -> bool {
        if index >= BOARD_SIZE {
            panic!("Bitboard::get() out of bounds")
        }
        (self.occupancy[index as usize / 63] & ((1 as u64) << (index % 63))) != 0
    }

    // returns precomputed result: if 3x3 block is captured/drawn
    pub fn block_marked(&self, block_i: u8) -> bool {
        (self.occupancy[1] | (1 << (18 + block_i))) != 0
    }
}

/*
Need 81 * 2 bits for each player's general board
4 bits for the last large square played
1 bit for the side to move

167 bits in total. 3 longs.
*/
#[derive(Copy, Clone)]
pub struct Position {
    bitboards: [Bitboard<Capturer>; 2],
    all_bitboard: Bitboard<Drawer>,
    to_move: Side,
    captured_occ: B33, // occupancy of captured blocks
    draw_occ: B33,     // occupancy of drawn blocks
    // index of
    last_block: u8,
}

impl Position {
    pub fn new() -> Position {
        Position {
            bitboards: [Bitboard::new(), Bitboard::new()],
            all_bitboard: Bitboard::new(),
            to_move: Side::X,
            captured_occ: 0,
            draw_occ: 0,
            last_block: 9,
        }
    }

    pub fn from_board(repr: &str) -> Position {
        assert!(repr.len() == 133);
        let mut pos = Position::new();
        let mut n_x = 0;
        let mut n_o = 0;
        for (i, c) in repr.chars().enumerate() {
            if i == 132 {
                // 11 * 12 + 1 (space) + 1 (block index) - 1
                if c == '-' {
                    pos.last_block = 9;
                } else {
                    pos.last_block = c as u8 - '0' as u8;
                }
                break;
            }
            if i % 4 == 3 || (i / 12) % 4 == 3 || i == 131 || i % 12 == 11 {
                continue;
            }
            if c == 'X' || c == 'O' {
                let col = match i % 12 {
                    0..=2 => i % 12,
                    4..=6 => i % 12 - 1,
                    8..=10 => i % 12 - 2,
                    _ => panic!("Bad index"),
                };
                let row = match i / 12 {
                    0..=2 => i / 12,
                    4..=6 => i / 12 - 1,
                    8..=10 => i / 12 - 2,
                    _ => panic!("Bad index"),
                };
                let index = to_bb_index!(row, col);
                let side = match c {
                    'X' => {
                        n_x += 1;
                        Side::X
                    }
                    'O' => {
                        n_o += 1;
                        Side::O
                    }
                    _ => panic!("Impossible"),
                };
                let own_bb = &mut pos.bitboards[side as usize];
                pos.captured_occ |= own_bb.set(index);
                // place piece on all_bitboard
                pos.draw_occ |= pos.all_bitboard.set(index);
            }
        }
        pos.to_move = match n_x - n_o {
            0 => Side::X,
            1 => Side::O,
            _ => panic!("Number of X and O not possible"),
        };
        return pos;
    }

    pub fn legal_moves(&self) -> Moves {
        let mut moves = Moves::new();
        if self.last_block == 9 || self.captured_occ & (1 << self.last_block) != 0 {
            // can go anywhere that is not captured
            let mut blocks: B33 = !self.captured_occ & BLOCK_OCC;
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

    fn add_block_moves(&self, block_i: u8, moves: &mut Moves) {
        let mut block_occ: B33 = !self.all_bitboard.block_occ(block_i) & BLOCK_OCC;
        let offset = block_i as Idx * 9;
        while block_occ != 0 {
            let idx = block_occ.tzcnt();
            moves.add(idx + offset);
            block_occ &= !(1 << idx);
        }
    }

    pub fn make_move(&mut self, index: Idx) {
        // place piece
        let own_bb = &mut self.bitboards[self.to_move as usize];
        let capture_update = own_bb.set(index);
        assert_eq!(self.captured_occ & capture_update, 0);
        self.captured_occ |= capture_update;

        // place piece on all_bitboard
        self.draw_occ |= self.all_bitboard.set(index);

        // update last_block
        self.last_block = to_local_index!(index);

        self.to_move = self.to_move.other();
    }

    pub fn unmake_move(&mut self, index: Idx, last_block: u8) {
        self.to_move = self.to_move.other();

        self.last_block = last_block;

        let own_bb = &mut self.bitboards[self.to_move as usize];
        self.captured_occ &= !own_bb.unset(index);

        self.draw_occ &= !self.all_bitboard.unset(index);
    }

    pub fn board_repr(&self) -> String {
        // 266 = 24 * 11 + 2
        // fill with space first
        let mut repr: [char; 266] = [' '; 266];

        // place pieces
        for row in 0..9 {
            let row_offset = row / 3;
            for col in 0..9 {
                let col_offset = col / 3;

                // compute index in output string
                let out_row = row + row_offset;
                let out_col = 2 * (col + col_offset) + 1;
                let out_ind = out_row * 24 + out_col;

                let index = to_bb_index!(row, col);

                if !self.all_bitboard.get(index) {
                    repr[out_ind as usize] = '-';
                } else if self.bitboards[Side::X as usize].get(index) {
                    repr[out_ind as usize] = 'X';
                } else {
                    repr[out_ind as usize] = 'O';
                }
            }
        }

        // place newlines
        for row in 0..10 {
            repr[23 + 24 * row] = '\n';
        }

        // place vertical bars
        for row in 0..11 {
            repr[24 * row + 7] = '|';
            repr[24 * row + 15] = '|';
        }

        // place horizontal bars
        for col in 0..23 {
            repr[24 * 3 + col] = '-';
            repr[24 * 7 + col] = '-';
        }

        repr[263] = match self.to_move {
            Side::X => 'X',
            Side::O => 'O',
        };
        repr[264] = match self.last_block {
            0..=8 => ('0' as u8 + self.last_block) as char,
            9 => '-',
            _ => panic!("last_block out of bounds: {}", self.last_block),
        };
        repr[265] = '\n';

        // for aesthetics
        repr[79] = '|';
        repr[183] = '|';

        return repr.iter().collect::<String>();
    }

    #[allow(dead_code)]
    pub fn assert(&self) {
        const B_18: u64 = 0x3FFFF;
        const B_27: u64 = 0x7FFFFFF;
        let x_occ0 = self.bitboards[Side::X as usize].occupancy[0];
        let x_occ1 = self.bitboards[Side::X as usize].occupancy[1];
        let x_big_occ = x_occ1 >> 18;
        let o_occ0 = self.bitboards[Side::O as usize].occupancy[0];
        let o_occ1 = self.bitboards[Side::O as usize].occupancy[1];
        let o_big_occ = o_occ1 >> 18;

        // occupancies don't overlap
        assert_eq!(x_occ0 & o_occ0, 0);
        assert_eq!((x_occ1 & o_occ1) & B_18, 0);

        // occupancies add up to all_bitboard
        assert_eq!(x_occ0 | o_occ0, self.all_bitboard.occupancy[0]);
        assert_eq!(
            (x_occ1 | o_occ1) & B_18,
            self.all_bitboard.occupancy[1] & B_18
        );

        // big occupancies don't overlap
        assert_eq!(x_big_occ & o_big_occ, 0);
        assert_eq!((x_big_occ | o_big_occ) as u16, self.captured_occ);

        // bit representations are within range
        assert_eq!(self.captured_occ & !BLOCK_OCC, 0);
        assert_eq!(self.draw_occ & !BLOCK_OCC, 0);
        assert_eq!(x_occ0 & (1 << 63), 0);
        assert_eq!(o_occ0 & (1 << 63), 0);
        assert_eq!(x_occ1 & !B_27, 0);
        assert_eq!(o_occ1 & !B_27, 0);
    }
}

pub fn perft(depth: u16, pos: &mut Position) -> u64 {
    pos.assert();
    if depth == 0 {
        return pos.legal_moves().size();
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

fn divide(depth: u16, pos: &mut Position) {
    pos.assert();

    let moves = pos.legal_moves();
    // special case; need to do this since perft doesn't output
    // 1 move for base case
    if depth == 0 {
        for mov in pos.legal_moves() {
            println!("{}: 1", mov);
        }
    } else {
        for mov in pos.legal_moves() {
            let last_block = pos.last_block;
            pos.make_move(mov);
            let count = perft(depth - 1, pos);
            pos.unmake_move(mov, last_block);
            println!("{}: {}", mov, count);
        }
    }
}
