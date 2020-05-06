/* For importing/exporting positions based on formats */

use crate::moves::*;
use std::i64;

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

impl Position {
    /* 
    An example compact board (including newlines)
     O..|XX.|...
     ...|.X.|...
     ...|X.O|...
     -----------
     ...|...|...
     O..|...|...
     ...|...|...
     -----------
     ...|...|...
     ...|...|...
     ...|...|... 0;
    */
    pub fn from_compact_board(repr: &str) -> Position {
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

                // place piece
                let own_bb = &mut pos.bitboards[side as usize];
                let bi = own_bb.set(to_bb_index!(row, col));
                let block_occ = own_bb.get_block(bi);
                // update full block
                pos.full_blocks |= (pos.is_block_full(bi) as B33) << bi;

                // update hopeless occ for the other player
                pos.hopeless_occ[side.other() as usize] |= (block_hopeless(block_occ) as B33) << bi;
            }
        }
        pos.to_move = match n_x - n_o {
            0 => Side::X,
            1 => Side::O,
            _ => panic!("Number of X and O not possible"),
        };
        return pos;
    }

    pub fn to_pretty_board(&self) -> String {
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
                if self.bitboards[Side::X as usize].get(index) {
                    repr[out_ind as usize] = 'X';
                } else if self.bitboards[Side::O as usize].get(index) {
                    repr[out_ind as usize] = 'O';
                } else {
                    repr[out_ind as usize] = '-';
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

    pub fn from_bgn(repr: &str) -> Position {
        let mut pos = Position::new();
        let mut tokens = repr.split_whitespace();
        let level = tokens.next();

        // only support level 2 ultimate tic-tac-toe
        assert_eq!(level, Some("2"));

        let x_board = tokens.next().expect("too few tokens!");
        let o_board = tokens.next().expect("too few tokens!");

        let focus_block = tokens.next().expect("too few tokens: need focus block");
        assert_eq!(focus_block.len(), 1);

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
        return pos;
    }

    pub fn to_bgn(&self) -> String {
        // TODO
        let x_board = self.to_bgn_str(Side::X);
        let o_board = self.to_bgn_str(Side::O);
        return format!("2 {} {} {}", x_board, o_board, self.last_block);
    }
    
    // initialize bitboard of side using repr, a string of blocks
    // represented by hex, separated by "/"
    fn init_bgn_bb(&mut self, side: Side, repr: &str) -> u32 {
        let bitboard = &mut self.bitboards[side as usize];
        let mut tokens = repr.split("/");
        
        let mut count: u32 = 0;
        for bi in 0..9 {
            let tok = tokens.next().expect("too few blocks given. 9 expected");
            let occ = i64::from_str_radix(tok.trim(), 16).expect("could not parse hex string") as B33;
            count += occ.count_ones();
            bitboard.set_block(bi, occ);
            // update full block
            if occ == BLOCK_OCC {
                self.full_blocks |= 1 << bi;
            }
            // update hopeless occ
            self.hopeless_occ[self.to_move as usize] |= (block_hopeless(occ) as B33) << bi;
        }
        
        return count;
    }

    fn to_bgn_str(&self, side: Side) -> String {
        let bitboard = self.bitboards[side as usize];
        let mut str_list = Vec::new();
        for bi in 0..9 {
            let occ = bitboard.get_block(bi);
            str_list.push(format!("{:x}", occ));
        }
        return str_list.join("/");
    }

    // comma separated list of moves
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
