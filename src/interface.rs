use std::io::{self, BufRead, Stdin};
use std::process::exit;
use std::str::SplitWhitespace;

use uttt::engine::best_move;
use uttt::moves::*;

struct GameContext {
    depth: u16,
    eval: i32,
    history: Vec<String>,
}

impl GameContext {
    fn new() -> GameContext {
        GameContext {
            depth: 7,
            eval: 1234567, // indicates not updated
            history: Vec::new(),
        }
    }
}

fn next_line(stdin: &mut Stdin) -> String {
    stdin
        .lock()
        .lines()
        .next()
        .expect("there was no next line")
        .expect("the line could not be read")
}

fn command_help(_: &mut SplitWhitespace, _: &mut Position, _: &mut GameContext) -> bool {
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
l                       Output move list history
v                       Print principal variation
";
    println!("{}", HELP_TEXT);
    return false;
}

fn command_print(_: &mut SplitWhitespace, pos: &mut Position, _: &mut GameContext) -> bool {
    println!("{}", pos.to_pretty_board());
    return false;
}

fn command_make_move(
    tokens: &mut SplitWhitespace,
    pos: &mut Position,
    context: &mut GameContext,
) -> bool {
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
    context.history.push(mov.to_string());
    return true;
}

fn command_depth(
    tokens: &mut SplitWhitespace,
    _: &mut Position,
    context: &mut GameContext,
) -> bool {
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
            context.depth = dep;
        }
        None => println!("{}", context.depth),
    }
    return false;
}

fn command_evaluate(_: &mut SplitWhitespace, _: &mut Position, context: &mut GameContext) -> bool {
    println!("{}", context.eval);
    return false;
}

fn command_list(_: &mut SplitWhitespace, _: &mut Position, context: &mut GameContext) -> bool {
    println!("{}", context.history.join(", "));
    return false;
}

fn command_pv(_: &mut SplitWhitespace, _: &mut Position, context: &mut GameContext) -> bool {
    println!(
        "{}",
        context
            .history
            .iter()
            .map(|i| i.to_string())
            .collect::<String>()
    );
    return false;
}

fn main() {
    // let mut pos = Position::from_bgn("2 45/1c4/1ac/90/12/11/38/24/28 92/20/43/105/125/cc/6/c8/6 7");
    // let mut pos = Position::from_move_list("40, 36, 0, 1, 11, 18, 6, 55, 17, 74, 21, 29, 23, 48, 34, \
    // 66, 31, 41, 45, 4, 37, 14, 49, 38, 26, 73, 15, 56, 25, 69, 59, 51, 57, 35, 77, 52, 65, 24, 58, 44, \
    // 75, 27, 2, 19, 16, 70, 68");
    // TODO remove last two moves and check how it gets screwed
    // let mut pos = Position::from_move_list("36, 0, 2, 18, 4, 37, 15, 55, 12, 29, 19, 11, 25, 66, 32, 48, 31,\
    //  40, 39, 30, 35, 74, 24, 58, 42, 61, 63, 5, 53, 80, 77, 45, 6, 14, 50, 47, 23, 46, 9, 75, 27, 20");
    let mut pos = Position::from_move_list(
        "36, 0, 2, 18, 4, 37, 15, 55, 12, 29, 19, 11, 25, 66, 32, 48, 31,\
    40, 39, 30, 35, 74, 24, 58, 42, 61, 63, 5, 53, 80, 77, 45, 6, 14, 50, 47, 23, 46, 9, 75",
    );
    // let mut pos = Position::new();
    let mut player_move = pos.side_to_move() == Side::X;
    let mut stdin = io::stdin();
    let mut context = GameContext::new();
    loop {
        let result = pos.get_result();
        match result {
            GameResult::XWon => {
                println!("X wins!");
                println!("History: {}", context.history.join(", "));
                return;
            }
            GameResult::OWon => {
                println!("O wins!");
                println!("History: {}", context.history.join(", "));
                return;
            }
            GameResult::Draw => {
                println!("It's a draw!");
                println!("History: {}", context.history.join(", "));
                return;
            }
            GameResult::Ongoing => {}
        }

        if player_move {
            let mut move_made = false;
            println!("{}", pos.to_bgn());
            println!("{}", pos.to_pretty_board());
            while !move_made {
                println!("Your move.");
                println!("Enter command. 'h' for help.");
                let line = next_line(&mut stdin);
                let mut tokens = line.split_whitespace();
                // function returns true if a move is made
                let func: fn(&mut SplitWhitespace, &mut Position, &mut GameContext) -> bool =
                    match tokens.next() {
                        Some("h") => command_help,
                        Some("p") => command_print,
                        Some("m") => command_make_move,
                        Some("q") => |_, _, _| exit(0),
                        Some("d") => command_depth,
                        Some("e") => command_evaluate,
                        Some("l") => command_list,
                        Some("v") => command_pv,
                        Some(_) => command_help,
                        None => |_, _, _| false,
                    };

                move_made = func(&mut tokens, &mut pos, &mut context);
            }

            player_move = false;
        } else {
            let idx;
            println!("Thinking...");
            let tup = best_move(context.depth, &mut pos);
            idx = tup.0;
            context.eval = tup.1;
            pos.make_move(idx);
            context.history.push(idx.to_string());
            println!("Your opponent played {} {}", idx / 9, idx % 9);
            println!();
            player_move = true;
        }
    }
}
