# Ultimate TicTacToe Engine

## Interface - Play against bot
`cargo run --release --bin interface`

## To bundle
Note that `bundle` is my fork of bundle, which does not automatically format and uses the first
bin target in Cargo.toml

Run `./scripts/bundle.sh` in a unix-based terminal

## TODOs

* quiet search - do quiet search if # of moves < N. (make it modal, i.e. once there is a capture, do the capture and all
moves that come after are captures). Before that, add mobility to eval
* table
* clean up some TODOs. Some of them are important
* some move ordering, by capturing a block if possible (is this a good idea? capturing blocks can be bad)
* Finish dead-drawn implementation