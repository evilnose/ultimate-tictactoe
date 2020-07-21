use std::sync::mpsc;
use std::collections::HashMap;

extern crate uttt;

use uttt::engine::*;
use uttt::engine::utils::*;
use uttt::moves::*;

fn main() {
    let mut nb_stdin = NonBlockingStdin::new();
    init_moves();
    init_engine();
    let mut client = Client::new();
    loop {
        client.tic();
        let line = nb_stdin.try_nextline();
        match line {
            Some(line) => {
                eprintln!("NOTE: received command: {}", line);
                let split = line.split_whitespace().collect::<Vec<&str>>();
                match split[0] {
                    "uti" => println!("utiok"),
                    "id" => println!("myid name=barbar;version=0.0.1"),
                    "pos" => client.handle_pos(split),
                    "search" => client.handle_search(split),
                    _ => eprintln!("unknown command: '{}'", split[0]),
                };
            },
            None => {}
        };
    }
}

struct Client {
    pos: Position,
    searching: bool,
    receiver: Option<mpsc::Receiver<SearchResult>>,
}

impl Client {
    fn new() -> Client {
        Client {
            pos: Position::new(),
            searching: false,
            receiver: None,
        }
    }

    // called every loop. Sends whatever needed to UTI 
    fn tic(&mut self) {
        if self.searching {
            // check if search finished
            match self.receiver.as_ref().unwrap().try_recv() {
                Ok(search_res) => {
                    self.searching = false;
                    eprintln!("NOTE: sending 'info best_move={}; eval={}'", search_res.best_move, search_res.eval);
                    println!("info best_move={}; eval={}", search_res.best_move, search_res.eval);
                },
                Err(mpsc::TryRecvError::Empty) => {},
                Err(mpsc::TryRecvError::Disconnected) => panic!("fatal: search channel disconnected"),
            }
        }
    }

    fn handle_pos(&mut self, split: Vec<&str>) {
        if split.len() < 2 {
            eprintln!("error: pos command needs a subcommand");
            return;
        }

        match split[1] {
            "start" => self.pos = Position::new(),
            "bgn" => {
                if split.len() < 3 {
                    eprintln!("error: need bgn string");
                    return;
                }
                self.pos = Position::from_bgn(split[2]);
            },
            "moves" => {
                for i in 2..split.len() {
                    self.pos.make_move(split[i].parse::<Idx>().unwrap());
                }
            },
            _ => {},
        }
    }

    fn handle_search(&mut self, split: Vec<&str>) {
        if self.searching {
            eprintln!("error: search in progress");
            return;
        }
        if split.len() < 2 {
            eprintln!("error: search needs a subcommand");
            return;
        }

        //let remaining = &split[2..].join("");
        match split[1] {
            "free" =>  {
                // TODO call parse_keyvalue and get xtime otime etc.
                if split.len() < 4 {
                    eprintln!("error: too few arguments for 'search free'");
                    return;
                }
                let xtime: u64 = split[2].parse().expect("'search free' <xtime> <otime>");
                let otime: u64 = split[2].parse().expect("'search free' <xtime> <otime>");

                let mut manager = Manager::from_position(self.pos);
                manager.search_free(xtime, otime);

                self.searching = true;
            },
            "depth" => {
                panic!("not implemented");
            },
            "time" => {
                panic!("not implemented");
            },
            "nodes" => {
                panic!("not implemented");
            },
            "forever" => {
                panic!("not implemented");
            },
            _ => eprintln!("error: unknown search subcommand"),
        }
    }
}

fn parse_keyvalue(line: &str) -> HashMap<&str, &str> {
    let mut map = HashMap::new();
    let tokens = line.split(";");
    for tok in tokens {
        //let tok = tok.trim();
        let kv = tok.split("=").collect::<Vec<&str>>();
        if kv.len() != 2 {
            eprintln!("error: key-value pair format incorrect");
            continue;
        }

        map.insert(kv[0].trim(), kv[1].trim());
    }
    map
}
