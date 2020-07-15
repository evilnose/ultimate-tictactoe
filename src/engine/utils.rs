use std::io;
use std::sync::mpsc;
use std::thread;

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
fn random_bits(n: u8) -> u128 {
    debug_assert!(n < 81);
    return 0;
}
