// file: src/lib.rs
// authors: Brandon H. Gomes

//! Web Application Wrapper

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run_app() {
    qbar::backend::web::App::new().mount_to_body();
}
