use std::io;
use std::sync::mpsc;
use std::thread;
use crate::engine::config::*;
use crate::moves::*;

const N_NATURAL_LOGS: usize = 80000;
static mut NATURAL_LOG_TABLE: [f32; N_NATURAL_LOGS] = [0.0; N_NATURAL_LOGS];

pub struct NonBlockingStdin {
    receiver: mpsc::Receiver<String>,
}

// NOTE (Gary) this should only be created once at the top of main and later
// passed to functions as they need it, since it doesn't make sense to
// have multiple of this (but Rust also doesn't play well with singletons)
impl NonBlockingStdin {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            tx.send(buf).unwrap();  // convert String to &str
        });

        Self {
            receiver: rx,
        }
    }

    pub fn try_nextline(&mut self) -> Option<String> {
        match self.receiver.try_recv() {
            Ok(val) => Some(val),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => panic!("stdin thread disconnected"),
        }
    }
}

// generate a random number with n random bits set in the lower 81 bits exactly.
// note: n must be less than 81
pub fn random_bits(n: u8) -> u128 {
    debug_assert!(n < 81);
    panic!("not implemmented");
}


#[inline(always)]
pub fn natural_log(x: u32) -> f32 {
    // TODO optimize. search "fast natural log"
    if x >= N_NATURAL_LOGS as u32 {
        return (x as f32).log(2.71828182845);
    }
    unsafe {
        return NATURAL_LOG_TABLE[x as usize];
    }
}

pub(crate) fn init_natural_log_table() {
    unsafe {
        NATURAL_LOG_TABLE[0] = 1.0;
        for i in 0..N_NATURAL_LOGS {
            NATURAL_LOG_TABLE[i] = (i as f32).log(2.71828182845);
        }
    }
}

// simple helper function that returns 1 if equal
// is true and -1 if not
#[inline(always)]
pub(crate) fn side_multiplier(side: Side) -> Score {
    (1 - 2 * (side as i32)) as Score
}

// on codingame, even when the board is filled, whoever has more blocks
// captured wins. This returns 1 if X has more captured, -1 if O does,
// and 0 if dead drawn
#[inline(always)]
pub(crate) fn codingame_drawn(pos: &Position) -> f32 {
    let diff = (pos.bitboards[0].n_captured() as i16) - (pos.bitboards[1].n_captured() as i16);
    return ((diff != 0) as i32 as f32) * (diff as f32).signum();
}
