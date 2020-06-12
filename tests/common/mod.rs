use uttt::moves;
use uttt::engine;

pub fn setup() {
    moves::init_moves();
    engine::init_engine();
}
