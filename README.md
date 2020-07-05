# Ultimate TicTacToe Engine

## Interface - Play against bot
`cargo run --release --bin interface`

## To bundle
Note that `bundle` is my fork of bundle, which does not automatically format and uses the first
bin target in Cargo.toml

* `bundle . > out.rs`
* `rustfmt out.rs`
* copy the following and paste it to the top of `out.rs`:
```
use crate::engine::*;
use crate::engine::config::*;
use crate::engine::eval::*;
use crate::engine::search::*;
use crate::engine::utils::*;
use crate::format::*;
use crate::moves::*;
```

## TODOs

* Finish dead-drawn implementation
* quiet search
* clean up some TODOs. Some of them are important
* some move ordering, by capturing a block if possible