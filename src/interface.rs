use std::io::{self, BufRead, Stdin};
use std::process::exit;
use std::str::SplitWhitespace;

use uttt::engine::best_move;
use uttt::moves::*;

static mut DEPTH: u16 = 5;
static mut EVAL: i32 = 0;

fn next_line(stdin: &mut Stdin) -> String {
    stdin
        .lock()
        .lines()
        .next()
        .expect("there was no next line")
        .expect("the line could not be read")
}

fn command_help(_: &mut SplitWhitespace, _: &mut Position) -> bool {
    static HELP_TEXT: &'static str = "
COMMANDS
========
h                       Display this message.
p                       Print current board.
m <i> <j>               Make move with big index i and small index j.
                            Big index is the row-major index of a 3x3 block;
                            Small index is the row-major index of a cell
                            in that block.
d [depth]               Change difficulty to the given depth. If no 
                            argument is given, the current depth is printed.
q                       Quit this program.
e                       Print evaluation score.
";
    println!("{}", HELP_TEXT);
    return false;
}

fn command_print(tokens: &mut SplitWhitespace, pos: &mut Position) -> bool {
    println!("{}", pos.to_pretty_board());
    return false;
}

fn command_make_move(tokens: &mut SplitWhitespace, pos: &mut Position) -> bool {
    let big_index = match tokens.next() {
        Some(val) => val,
        None => {
            println!("ERROR: Need 2 arguments!");
            return false;
        }
    };
    let small_index = match tokens.next() {
        Some(val) => val,
        None => {
            println!("ERROR: Need 2 arguments!");
            return false;
        }
    };

    // parse into ints
    let big_index: u32 = match big_index.parse() {
        Ok(val) => val,
        Err(err) => {
            println!("ERROR parsing index: {:?}", err);
            return false;
        }
    };
    let small_index: u32 = match small_index.parse() {
        Ok(val) => val,
        Err(err) => {
            println!("ERROR parsing index: {:?}", err);
            return false;
        }
    };

    if big_index >= 9 || small_index >= 9 {
        println!("move index out of bounds! (0-8)");
    }

    let mov = (big_index * 9 + small_index) as Idx;

    if !pos.legal_moves().contains(mov) {
        println!("ERROR: illegal move");
        return false;
    }

    pos.make_move(mov);
    return true;
}

fn command_depth(tokens: &mut SplitWhitespace, _: &mut Position) -> bool {
    match tokens.next() {
        Some(tok) => {
            let dep: u16 = match tok.parse() {
                Ok(val) => val,
                Err(err) => {
                    println!("ERROR parsing depth: {:?}", err);
                    return false;
                }
            };
            if dep == 0 {
                println!("ERROR: depth must be >= 1");
                return false;
            }
            unsafe {
                DEPTH = dep;
            }
        }
        None => unsafe {
            println!("{}", DEPTH);
        },
    }
    return false;
}

fn command_evaluate(_: &mut SplitWhitespace, _: &mut Position) -> bool {
    unsafe {
        println!("{}", EVAL);
    }
    return false;
}

fn main() {
    let mut pos = Position::from_bgn("2 92/14/30/c0/10/140/6/4/0 1/3/c4/0/7/8/48/a0/20 7");
    let mut player_move = pos.side_to_move() == Side::X;
    let mut stdin = io::stdin();
    loop {
        let result = pos.get_result();
        match result {
            GameResult::XWon => {
                println!("X wins!");
                return;
            }
            GameResult::OWon => {
                println!("O wins!");
                return;
            }
            GameResult::Draw => {
                println!("It's a draw!");
                return;
            }
            GameResult::Ongoing => {}
        }

        if player_move {
            let mut move_made = false;
            println!("{}", pos.to_pretty_board());
            while !move_made {
                println!("Your move.");
                println!("Enter command. 'h' for help.");
                let line = next_line(&mut stdin);
                let mut tokens = line.split_whitespace();
                // function returns true if a move is made
                let func: fn(&mut SplitWhitespace, &mut Position) -> bool = match tokens.next() {
                    Some("h") => command_help,
                    Some("p") => command_print,
                    Some("m") => command_make_move,
                    Some("q") => |_, _| exit(0),
                    Some("d") => command_depth,
                    Some("e") => command_evaluate,
                    None => |_, _| false,
                    Some(_) => command_help,
                };

                move_made = func(&mut tokens, &mut pos);
            }

            player_move = false;
        } else {
            let idx;
            println!("Thinking...");
            unsafe {
                let tup = best_move(DEPTH, &mut pos);
                idx = tup.0;
                EVAL = tup.1;
            }
            pos.make_move(idx);
            println!("Your opponent played {} {}", idx / 9, idx % 9);
            println!();
            player_move = true;
        }
    }
}
