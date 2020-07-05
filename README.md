# Ultimate TicTacToe Engine

## Interface - Play against bot
`cargo run --release --bin interface`

## To bundle
Note that `bundle` is my fork of bundle, which does not automatically format and uses the first
bin target in Cargo.toml

Run `./scripts/bundle.sh` in a unix-based terminal

## TODOs

* Finish dead-drawn implementation
* quiet search (quiet search idea. match small board with big board -- check if there are forced moves)
* clean up some TODOs. Some of them are important
* some move ordering, by capturing a block if possible (is this a good idea? capturing blocks can be bad)