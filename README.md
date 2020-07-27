# Ultimate TicTacToe Engine

## Status
Right now the MCTS bot is strongest, at least for Codingame time control of 100ms per move. It is in the `mcts.rs` file.

## Interface - Play against bot
`cargo run --release --bin interface`

## To bundle
Note that `bundle` is my fork of bundle, which does not automatically format and uses the first
bin target in Cargo.toml

Run `./scripts/bundle.sh` in a unix-based terminal

## TODOs

* table
* clean up some TODOs. Some of them are important
* some move ordering, by capturing a block if possible (is this a good idea? capturing blocks can be bad)
* Finish dead-drawn implementation
