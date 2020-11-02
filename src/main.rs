// file: src/main.rs
// authors: Brandon H. Gomes

use {
    qbar::app,
    std::{env, process},
};

pub fn main() {
    process::exit(app::run(env::args().skip(1)).report())
}
