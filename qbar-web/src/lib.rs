// file: src/lib.rs
// authors: Brandon H. Gomes

//! Web Application

pub mod model;
pub mod term;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::prelude::*;
use yew::App;

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<model::Model>::new().mount_to_body();
}
