use crate::engine::*;
use crate::engine::config::*;
use crate::engine::eval::*;
use crate::engine::utils::*;
use crate::format::*;
use crate::moves::*;
use std::io::{self, BufRead};
pub mod moves {
    use std::iter::Peekable;
    use std::slice::Iter;
    pub(crate) type B33 = u16;
    pub type Idx = u8;
    const BOARD_SIZE: Idx = 81;
    pub const NULL_IDX: Idx = 88;
    macro_rules! to_local_index {
        ( $ index : expr ) => {{
            ($index % 9) as u8
        }};
    }
    #[derive(Copy, Clone)]
    pub struct BlockState(u8);
    impl BlockState {
        pub(crate) fn new(min_needed: u8, n_routes: u8) -> BlockState {
            debug_assert!(min_needed == 4 || n_routes >= 1);
            debug_assert!(min_needed <= 4);
            BlockState(min_needed | n_routes << 3)
        }
        pub(crate) fn min_needed(&self) -> u8 {
            self.0 & 7
        }
        pub(crate) fn n_routes(&self) -> u8 {
            self.0 >> 3
        }
    }
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
    pub(crate) const N_BLOCK33: usize = 262144;
    static mut BLOCK_STATE_TABLE: [BlockState; N_BLOCK33] = [BlockState(0); N_BLOCK33];
    static mut INITIALIZED: bool = false;
    pub fn init_moves() {
        for idx in 0..N_BLOCK33 {
            let my_occ = idx as B33 & BLOCK_OCC;
            let their_occ = (idx >> 9) as B33 & BLOCK_OCC;
            let mut counts: [u8; 5] = [0; 5];
            let mut min_count = 4;
            for win_occ in WIN_OCC_LIST.iter() {
                if their_occ & win_occ != 0 {
                    continue;
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
        get_block_state(occ, 0).min_needed() == 0
    }
    pub(crate) fn get_block_hopeless(occ: B33) -> bool {
        debug_assert_eq!(occ & !BLOCK_OCC, 0);
        get_block_state(0, occ).min_needed() == 4
    }
    #[inline(always)]
    fn bool_to_block(filled: bool) -> u128 {
        ((0i128 - (filled as i128)) & BLOCK_OCC_I128) as u128
    }
    #[derive(Copy, Clone)]
    pub struct Moves(u128);
    impl Moves {
        #[inline(always)]
        pub fn size(&self) -> usize {
            self.0.count_ones() as usize
        }
        #[inline(always)]
        pub fn contains(&self, index: Idx) -> bool {
            self.0 & (1u128 << index) != 0
        }
        #[inline(always)]
        pub fn add(&mut self, index: Idx) {
            self.0 |= 1u128 << index;
        }
        #[inline(always)]
        pub fn remove(&mut self, index: Idx) {
            self.0 &= !(1u128 << index);
        }
        #[inline(always)]
        pub fn intersect(&self, other: Moves) -> Moves {
            Moves(self.0 & other.0)
        }
        #[inline(always)]
        pub fn subtract(&self, other: Moves) -> Moves {
            Moves(self.0 & !other.0)
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
        pub(crate) fn set(&mut self, index: Idx) -> u8 {
            debug_assert!(index < BOARD_SIZE);
            self.0 |= 1u128 << index;
            let block_i = index as u8 / 9;
            let block = self.get_block(block_i);
            let won = get_block_won(block);
            self.0 |= (won as u128) << (BOARD_SIZE + block_i);
            self.0 |= bool_to_block(won) << (block_i * 9);
            return block_i;
        }
        pub fn get(&self, index: Idx) -> bool {
            debug_assert!(index < BOARD_SIZE);
            self.0 & ((1 as u128) << index) != 0
        }
        pub fn set_block(&mut self, block_i: u8, occ: B33) {
            debug_assert!(block_i < 9);
            debug_assert_eq!(occ & !BLOCK_OCC, 0);
            self.0 |= (occ as u128) << (block_i * 9);
            let won = get_block_won(occ);
            self.0 |= (won as u128) << (BOARD_SIZE + block_i);
            self.0 |= bool_to_block(won) << block_i * 9;
        }
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
        pub fn legal_moves(&self) -> Moves {
            debug_assert!(!self.is_over());
            let total_occ = self.bitboards[0].0 | self.bitboards[1].0;
            let full_board = self.last_block == ANY_BLOCK
                || (((total_occ >> (self.last_block * 9)) as B33) & BLOCK_OCC == BLOCK_OCC);
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
        #[inline(always)]
        pub fn is_hopeless(&self) -> bool {
            get_block_hopeless(self.hopeless_occ[0]) && get_block_hopeless(self.hopeless_occ[1])
        }
        #[inline(always)]
        pub fn is_drawn(&self) -> bool {
            (self.bitboards[0].0 | self.bitboards[1].0) & BOARD_OCC == BOARD_OCC
        }
        #[inline(always)]
        pub fn is_over(&self) -> bool {
            self.is_won(Side::X) || self.is_won(Side::O) || self.is_drawn()
        }
        pub fn make_move(&mut self, index: Idx) {
            let own_bb = &mut self.bitboards[self.to_move as usize];
            let bi = own_bb.set(index);
            let block_occ = own_bb.get_block(bi);
            self.to_move = self.to_move.other();
            self.hopeless_occ[self.to_move as usize] |=
                (get_block_hopeless(block_occ) as B33) << bi;
            self.last_block = to_local_index!(index);
        }
        #[allow(dead_code)]
        pub fn assert(&self) -> bool {
            debug_assert_eq!(self.bitboards[0].0 >> (BOARD_SIZE + 9), 0);
            debug_assert_eq!(self.bitboards[1].0 >> (BOARD_SIZE + 9), 0);
            return true;
        }
        pub fn cur_ply(&self) -> u16 {
            (self.bitboards[0].0 | self.bitboards[1].0).count_ones() as u16
        }
        #[inline]
        pub fn get_1occ(&self, side: Side) -> Moves {
            let mut ret = Moves(0);
            let mut my_occ = self.bitboards[side as usize].0;
            let mut their_occ = self.bitboards[side as usize].0;
            while my_occ != 0 {
                let my_block = my_occ & BLOCK_OCC as u128;
                let their_block = their_occ & BLOCK_OCC as u128;
                let bstate = get_block_state(my_block as B33, their_block as B33);
                if bstate.min_needed() == 1 {
                    let block_moves = Moves(my_block);
                    for mov in block_moves {
                        if get_block_won((my_block | (1 << mov)) as B33) {
                            ret.add(mov);
                        }
                    }
                }
                my_occ >>= 9;
                their_occ >>= 9;
            }
            return ret;
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
        if depth == 0 {
            for mov in pos.legal_moves() {
                println!("{}: 1", mov);
            }
        } else {
            for mov in pos.legal_moves() {
                let mut temp = pos.clone();
                temp.make_move(mov);
                let count = perft(depth - 1, &mut temp);
                println!("{}: {}", mov, count);
            }
        }
    }
    #[allow(dead_code)]
    pub fn perft_with_progress(depth: u16, pos: &mut Position) {
        debug_assert!(pos.assert());
        let moves = pos.legal_moves();
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
        use super::*;
        #[test]
        fn test_blockwon() {
            init_moves();
            assert!(get_block_won(0b111111111));
            assert!(get_block_won(0b111000000));
            assert!(!get_block_won(0b000000000));
        }
    }
}
pub mod format {
    use crate::moves::*;
    use std::i64;
    macro_rules! to_bb_index {
        ( $ row : expr , $ col : expr ) => {{
            let row: usize = $row;
            let col: usize = $col;
            let bi = ((row / 3) * 3 + (col / 3));
            let small_row = row % 3;
            let small_col = col % 3;
            (bi * 9 + (small_row * 3 + small_col)) as Idx
        }};
    }
    impl Position {
        pub fn from_compact_board(repr: &str, to_move: Side, auto_side: bool) -> Position {
            let repr = repr.trim();
            let repr = repr.replace("\r\n", "\n");
            assert!(repr.len() == 133);
            let mut pos = Position::new();
            let mut n_x = 0;
            let mut n_o = 0;
            for (i, c) in repr.chars().enumerate() {
                if i == 132 {
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
                        0...2 => i % 12,
                        4...6 => i % 12 - 1,
                        8...10 => i % 12 - 2,
                        _ => panic!("Bad index"),
                    };
                    let row = match i / 12 {
                        0...2 => i / 12,
                        4...6 => i / 12 - 1,
                        8...10 => i / 12 - 2,
                        _ => panic!("Bad index"),
                    };
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
                    let bi = own_bb.set(to_bb_index!(row, col));
                    let block_occ = own_bb.get_block(bi);
                    pos.hopeless_occ[side.other() as usize] |=
                        (get_block_hopeless(block_occ) as B33) << bi;
                }
            }
            if auto_side {
                pos.to_move = match n_x - n_o {
                    0 => Side::X,
                    1 => Side::O,
                    _ => panic!("Number of X and O not possible"),
                };
            } else {
                pos.to_move = to_move;
            }
            return pos;
        }
        pub fn to_pretty_board(&self) -> String {
            let mut repr: [char; 266] = [' '; 266];
            for row in 0..9 {
                let row_offset = row / 3;
                for col in 0..9 {
                    let col_offset = col / 3;
                    let out_row = row + row_offset;
                    let out_col = 2 * (col + col_offset) + 1;
                    let out_ind = out_row * 24 + out_col;
                    let index = to_bb_index!(row, col);
                    if self.bitboards[Side::X as usize].get(index) {
                        repr[out_ind as usize] = 'X';
                    } else if self.bitboards[Side::O as usize].get(index) {
                        repr[out_ind as usize] = 'O';
                    } else {
                        repr[out_ind as usize] = '-';
                    }
                }
            }
            for row in 0..10 {
                repr[23 + 24 * row] = '\n';
            }
            for row in 0..11 {
                repr[24 * row + 7] = '|';
                repr[24 * row + 15] = '|';
            }
            for col in 0..23 {
                repr[24 * 3 + col] = '-';
                repr[24 * 7 + col] = '-';
            }
            repr[263] = match self.to_move {
                Side::X => 'X',
                Side::O => 'O',
            };
            repr[264] = match self.last_block {
                0...8 => ('0' as u8 + self.last_block) as char,
                9 => '-',
                _ => panic!("last_block out of bounds: {}", self.last_block),
            };
            repr[265] = '\n';
            repr[79] = '|';
            repr[183] = '|';
            return repr.iter().collect::<String>();
        }
        pub fn from_bgn(repr: &str) -> Position {
            let mut pos = Position::new();
            let mut tokens = repr.split_whitespace();
            let level = tokens.next();
            assert_eq!(level, Some("2"));
            let x_board = tokens.next().expect("too few tokens!");
            let o_board = tokens.next().expect("too few tokens!");
            let focus_block = tokens.next().expect("too few tokens: need focus block");
            assert_eq!(focus_block.len(), 1);
            let to_move = tokens.next().expect("too few tokens: need side to move");
            assert_eq!(to_move.len(), 1);
            let x_count = pos.init_bgn_bb(Side::X, x_board);
            let o_count = pos.init_bgn_bb(Side::O, o_board);
            if x_count == o_count {
                pos.to_move = Side::X;
            } else if x_count - o_count == 1 {
                pos.to_move = Side::O;
            } else {
                panic!("incorrect number of X/O pieces!");
            }
            pos.last_block = focus_block.parse().unwrap();
            pos.to_move = match to_move {
                "X" => Side::X,
                "O" => Side::O,
                other => panic!("to_move must be 'X' or 'O', but got '{}' instead", other),
            };
            return pos;
        }
        pub fn to_bgn(&self) -> String {
            let x_board = self.to_side_bgn(Side::X);
            let o_board = self.to_side_bgn(Side::O);
            let to_move = match self.to_move {
                Side::X => "X",
                Side::O => "O",
            };
            return format!("2 {} {} {} {}", x_board, o_board, self.last_block, to_move);
        }
        fn to_side_bgn(&self, side: Side) -> String {
            let bitboard = self.bitboards[side as usize];
            let mut str_list = Vec::new();
            for bi in 0..9 {
                let occ = bitboard.get_block(bi);
                str_list.push(format!("{:x}", occ));
            }
            return str_list.join("/");
        }
        fn init_bgn_bb(&mut self, side: Side, repr: &str) -> u32 {
            let bitboard = &mut self.bitboards[side as usize];
            let mut tokens = repr.split("/");
            let mut count: u32 = 0;
            for bi in 0..9 {
                let tok = tokens.next().expect("too few blocks given. 9 expected");
                let occ =
                    i64::from_str_radix(tok.trim(), 16).expect("could not parse hex string") as B33;
                count += occ.count_ones();
                bitboard.set_block(bi, occ);
                self.hopeless_occ[self.to_move as usize] |= (get_block_hopeless(occ) as B33) << bi;
            }
            return count;
        }
        pub fn from_move_list(repr: &str) -> Position {
            let mut pos = Position::new();
            let tokens = repr.split(",");
            for tok in tokens {
                let tok = tok.trim();
                pos.make_move(tok.parse::<Idx>().unwrap());
            }
            return pos;
        }
    }
}
pub mod engine {
    pub mod config {
        use crate::moves::*;
        pub type Score = f32;
        pub(crate) const SCORE_LOSS: f32 = -1e6;
        pub(crate) const SCORE_WIN: f32 = 1e6;
        pub(crate) const SCORE_NEG_INF: f32 = -1e7;
        pub(crate) const SCORE_POS_INF: f32 = 1e7;
        pub(crate) const MAX_SEARCH_PLIES: u16 = 40;
    }
    pub mod eval {
        use crate::engine::config::*;
        use crate::moves::*;
        pub type EvalFn = fn(&Position) -> Score;
        static SC_BLOCK_WON: Score = 8.0;
        static SC_NEED_1: Score = 3.0;
        static SC_NEED_2: Score = 0.5;
        static SC_NEED_3: Score = 0.1;
        static SC_HOPELESS: Score = 0.0;
        static SUBLINEAR_5: [Score; 10] = [1.0, 1.4, 1.7, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0];
        static mut BLOCK_SCORE_TABLE: [f32; N_BLOCK33] = [0.0; N_BLOCK33];
        pub fn init_block_score_table() {
            unsafe {
                for idx in 0..N_BLOCK33 {
                    let bs = get_block_state_by_idx(idx);
                    BLOCK_SCORE_TABLE[idx] = match bs.min_needed() {
                        0 => SC_BLOCK_WON,
                        1 => SC_NEED_1 * SUBLINEAR_5[bs.n_routes() as usize],
                        2 => SC_NEED_2 * SUBLINEAR_5[bs.n_routes() as usize],
                        3 => SC_NEED_3,
                        4 => SC_HOPELESS,
                        _ => panic!("min_needed is not in range [0, 4]"),
                    };
                }
            }
        }
        #[inline(always)]
        pub fn eval_block(x_occ: B33, o_occ: B33) -> Score {
            debug_assert!(
                get_block_won(o_occ)
                    || get_block_won(x_occ)
                    || ((x_occ | o_occ) == (x_occ + o_occ))
            );
            unsafe {
                BLOCK_SCORE_TABLE[(x_occ | (o_occ << 9)) as usize]
                    - BLOCK_SCORE_TABLE[(o_occ | (x_occ << 9)) as usize]
            }
        }
        pub fn eval(pos: &Position) -> Score {
            let side2move = side_multiplier(pos.to_move);
            let mut ret: Score = 0.0;
            for bi in 0..9 {
                ret += eval_block(
                    pos.bitboards[0].get_block(bi),
                    pos.bitboards[1].get_block(bi),
                );
            }
            let big_score = eval_block(
                pos.bitboards[0].captured_occ(),
                pos.bitboards[1].captured_occ(),
            );
            ret += big_score * 10.0;
            let mut mobility = pos.legal_moves().size() as Score / 2.0;
            if mobility > 5.0 {
                mobility = 5.0;
            }
            ret += mobility;
            return ret * side2move;
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
        #[inline(always)]
        pub(crate) fn side_multiplier(side: Side) -> Score {
            (1 - 2 * (side as i32)) as Score
        }
    }
    pub mod utils {
        use std::io;
        use std::sync::mpsc;
        use std::thread;
        pub struct NonBlockingStdin {
            receiver: mpsc::Receiver<String>,
        }
        impl NonBlockingStdin {
            pub fn new() -> Self {
                let (tx, rx) = mpsc::channel();
                thread::spawn(move || loop {
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf).unwrap();
                    tx.send(buf).unwrap();
                });
                Self { receiver: rx }
            }
            pub fn try_nextline(&mut self) -> Option<String> {
                match self.receiver.try_recv() {
                    Ok(val) => Some(val),
                    Err(mpsc::TryRecvError::Empty) => None,
                    Err(mpsc::TryRecvError::Disconnected) => panic!("stdin thread disconnected"),
                }
            }
        }
        fn random_bits(n: u8) -> u128 {
            debug_assert!(n < 81);
            return 0;
        }
    }
    use crate::engine::config::*;
    use crate::engine::eval::*;
    use crate::moves::*;
    use std::fmt;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::Duration;
    struct StopSearch;
    impl fmt::Debug for StopSearch {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "The search has been signaled to stop. This is used internally by alpha_beta_dfs()"
            )
        }
    }
    struct SearchState {
        nodes_searched: u64,
    }
    impl SearchState {
        fn new(alloc_millis: u64) -> SearchState {
            SearchState { nodes_searched: 0 }
        }
    }
    pub struct SearchResult {
        pub best_move: Idx,
        pub eval: Score,
    }
    pub struct Manager {
        position: Position,
    }
    impl Manager {
        pub fn from_position(pos: Position) -> Manager {
            Manager { position: pos }
        }
        pub fn search_fixed_time(&self, alloc_millis: u64) -> SearchResult {
            let (tx, rx) = mpsc::channel();
            let localpos = self.position;
            let stop_search = Arc::new(AtomicBool::new(false));
            let localstop = Arc::clone(&stop_search);
            thread::spawn(move || {
                let mut worker = Worker::new(localpos, tx.clone(), localstop);
                worker.search_fixed_time(alloc_millis);
            });
            assert!(alloc_millis > 30);
            thread::sleep(Duration::from_millis(alloc_millis - 25));
            stop_search.swap(true, Ordering::Relaxed);
            if let Ok(res) = rx.recv() {
                return res;
            } else {
                panic!("recv failed!");
            }
        }
        pub fn search_free(&mut self, x_millis: u64, o_millis: u64) {
            let my_millis = match self.position.to_move {
                Side::X => x_millis,
                Side::O => o_millis,
            };
            let cur_ply = self.position.cur_ply();
            debug_assert!(cur_ply < 81);
            let alloc_millis: u128;
            if cur_ply > 60 {
                alloc_millis = std::cmp::max(1000, my_millis as u128 / 3);
            } else {
                alloc_millis = (my_millis as f32 / (65.0 - cur_ply as f32)) as u128;
            }
            eprintln!(
                "NOTE: secs remaining: {}; allocated {}",
                my_millis as f32 / 1000.0,
                alloc_millis as f32 / 1000.0
            );
            self.search_fixed_time(alloc_millis as u64);
        }
    }
    struct Worker {
        position: Position,
        eval_fn: EvalFn,
        tx: mpsc::Sender<SearchResult>,
        stop: Arc<AtomicBool>,
    }
    impl Worker {
        pub fn new(pos: Position, tx: mpsc::Sender<SearchResult>, stop: Arc<AtomicBool>) -> Worker {
            Worker {
                position: pos,
                eval_fn: eval,
                tx: tx,
                stop: stop,
            }
        }
        fn search_till_depth(&mut self, depth: u16) -> Score {
            let mut state = SearchState::new(std::u64::MAX);
            return self
                .alpha_beta_dfs(
                    depth,
                    self.position,
                    SCORE_NEG_INF,
                    SCORE_POS_INF,
                    &mut state,
                )
                .expect("This should not happen, since infinite time was allocated");
        }
        pub fn search_fixed_time(&mut self, alloc_millis: u64) {
            let mut state = SearchState::new(alloc_millis);
            let moves = self.position.legal_moves();
            let mut t_moves = moves.clone();
            let mut best = t_moves.next().expect("error: no legal moves");
            let mut best_score = SCORE_NEG_INF;
            for depth in 1..=MAX_SEARCH_PLIES {
                let mut localmoves = moves.clone();
                {
                    let mut localpos = self.position.clone();
                    localpos.make_move(best);
                    let res = self.alpha_beta_dfs(
                        depth - 1,
                        localpos,
                        SCORE_NEG_INF,
                        SCORE_POS_INF,
                        &mut state,
                    );
                    best_score = match res {
                        Ok(s) => -s,
                        Err(_e) => {
                            eprintln!("NOTE: stopping at move 1");
                            self.tx
                                .send(SearchResult {
                                    best_move: best,
                                    eval: best_score,
                                })
                                .unwrap();
                            return;
                        }
                    };
                    localmoves.remove(best);
                }
                let mut move_idx = 1;
                for mv in localmoves {
                    move_idx += 1;
                    let mut localpos = self.position.clone();
                    localpos.make_move(mv);
                    let res = self.alpha_beta_dfs(
                        depth - 1,
                        localpos,
                        SCORE_NEG_INF,
                        -best_score,
                        &mut state,
                    );
                    let score = match res {
                        Ok(s) => -s,
                        Err(_e) => {
                            eprintln!("NOTE: stopping at move {}", move_idx);
                            self.tx
                                .send(SearchResult {
                                    best_move: best,
                                    eval: best_score,
                                })
                                .unwrap();
                            return;
                        }
                    };
                    if score > best_score {
                        best_score = score;
                        best = mv;
                    }
                }
                eprintln!("NOTE: depth {}/best {}/eval {}/", depth, best, best_score);
            }
            eprintln!("NOTE: stopping search since MAX_SEARCH_PLIES exceeded");
            self.tx
                .send(SearchResult {
                    best_move: best,
                    eval: best_score,
                })
                .unwrap();
        }
        fn alpha_beta_dfs(
            &mut self,
            depth: u16,
            pos: Position,
            alpha: Score,
            beta: Score,
            state: &mut SearchState,
        ) -> Result<Score, StopSearch> {
            debug_assert!(pos.assert());
            if pos.is_won(pos.to_move.other()) {
                state.nodes_searched += 1;
                return self.check_time(SCORE_LOSS);
            } else if pos.is_drawn() {
                state.nodes_searched += 1;
                let diff =
                    pos.bitboards[0].n_captured() as i16 - pos.bitboards[1].n_captured() as i16;
                if diff != 0 {
                    let sign = (diff as f32).signum();
                    return self.check_time(sign * side_multiplier(pos.to_move) * SCORE_WIN);
                } else {
                    return self.check_time(0.0);
                }
            } else if depth == 0 {
                state.nodes_searched += 1;
                let f = self.eval_fn;
                let my_1occ = self.position.get_1occ(self.position.to_move);
                let their_1occ = self.position.get_1occ(self.position.to_move.other());
                return self.quiesce_search(pos, my_1occ, their_1occ, f);
            }
            let moves = pos.legal_moves();
            let mut alpha = alpha;
            let my_1occ = pos.get_1occ(pos.to_move);
            let captures = moves.intersect(my_1occ);
            let moves = moves.subtract(captures);
            for mov in captures {
                let mut temp = pos.clone();
                temp.make_move(mov);
                let score = -self.alpha_beta_dfs(depth - 1, temp, -beta, -alpha, state)?;
                if score >= beta {
                    return Ok(beta);
                }
                if score > alpha {
                    alpha = score;
                }
            }
            for mov in moves {
                let mut temp = pos.clone();
                temp.make_move(mov);
                let score = -self.alpha_beta_dfs(depth - 1, temp, -beta, -alpha, state)?;
                if score >= beta {
                    return Ok(beta);
                }
                if score > alpha {
                    alpha = score;
                }
            }
            return Ok(alpha);
        }
        #[inline(always)]
        fn check_time(&mut self, eval: Score) -> Result<Score, StopSearch> {
            if self.stop.load(Ordering::Relaxed) {
                return Err(StopSearch);
            }
            return Ok(eval);
        }
        #[inline(always)]
        fn quiesce_search(
            &mut self,
            pos: Position,
            my_1occ: Moves,
            their_1occ: Moves,
            eval_fn: EvalFn,
        ) -> Result<Score, StopSearch> {
            let captures = pos.legal_moves().intersect(my_1occ);
            if captures.size() != 0 {
                let mut best = SCORE_NEG_INF;
                for mov in captures {
                    let mut temp = pos.clone();
                    temp.make_move(mov);
                    let my_1occ = self.position.get_1occ(self.position.to_move);
                    let their_1occ = self.position.get_1occ(self.position.to_move.other());
                    let score = -self.quiesce_search(temp, my_1occ, their_1occ, eval_fn)?;
                    if score > best {
                        best = score;
                    }
                }
                return self.check_time(best);
            } else {
                return self.check_time(eval_fn(&pos));
            }
        }
    }
    pub fn init_engine() {
        init_block_score_table();
    }
}
use std::time::Instant;
macro_rules! parse_input {
    ( $ x : expr , $ t : ident ) => {
        $x.trim().parse::<$t>().unwrap()
    };
}
fn next_line() -> String {
    io::stdin()
        .lock()
        .lines()
        .next()
        .expect("there was no next line")
        .expect("the line could not be read")
}
fn main() {
    init_moves();
    init_engine();
    let mut pos = Position::new();
    loop {
        let line = next_line();
        let inputs = line.split(" ").collect::<Vec<_>>();
        let opp_row = parse_input!(inputs[0], i32);
        let opp_col = parse_input!(inputs[1], i32);
        let line = next_line();
        let valid_action_count = parse_input!(line, i32);
        for _i in 0..valid_action_count as usize {
            next_line();
        }
        if opp_row != -1 {
            let index = (opp_col / 3) * 9 + (opp_col % 3) + (opp_row / 3) * 27 + 3 * (opp_row % 3);
            pos.make_move(index as u8);
        }
        let now = Instant::now();
        let manager = Manager::from_position(pos);
        let res = manager.search_fixed_time(100);
        let elapsed = now.elapsed();
        let idx = res.best_move;
        let col = ((idx / 9) % 3) * 3 + (idx % 3);
        let row = ((idx / 9) / 3) * 3 + (idx % 9) / 3;
        pos.make_move(idx);
        println!("{} {} {}", row, col, res.eval,);
    }
}
