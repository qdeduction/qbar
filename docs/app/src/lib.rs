// file: src/lib.rs
// authors: Brandon H. Gomes

//! QBar Web Application

use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

///
///
pub struct Model {
    link: ComponentLink<Self>,
    state: State,
}

///
///
pub struct State {
    entries: Vec<String>,
    value: String,
}

impl State {
    ///
    ///
    pub fn new() -> Self {
        State {
            entries: Vec::new(),
            value: String::new(),
        }
    }
}

impl Model {
    ///
    ///
    pub fn view_input(&self) -> Html {
        html! {
            <input class="cmdline"
                   value=&self.state.value
                   oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                   onkeypress=self.link.callback(|e: KeyboardEvent| {
                       if e.key() == "Enter" { Msg::Run } else { Msg::None }
                   }) />
        }
    }

    ///
    ///
    pub fn view_screen(&self) -> Html {
        html! {
            <div class="screen-wrapper">
                <ul class="screen">
                    { for self.state.entries.iter().map(|e| Self::view_entry(e)) }
                </ul>
            </div>
        }
    }

    ///
    ///
    pub fn view_entry(entry: &str) -> Html {
        html! {
            <li class="command"> { entry } </li>
        }
    }

    ///
    ///
    pub fn view_header(&self) -> Html {
        html! {
            <header>
                <a target="_blank" rel="noreferrer" href="https://qbar.io">
                    <img id="logo" type="image/svg" alt="qbar logo" src="../image/icon_trimmed.svg"/>
                </a>
            </header>
        }
    }

    ///
    ///
    pub fn view_footer(&self) -> Html {
        html! {
            <footer>
                {"2020 © "}
                <a class="footer-link" target="_blank" rel="noreferrer" href="https://bhgomes.dev">
                    {"Brandon H. Gomes"}
                </a>
            </footer>
        }
    }
}

///
///
pub enum Msg {
    ///
    ///
    Update(String),

    ///
    ///
    Clear,

    ///
    ///
    Up,

    ///
    ///
    Down,

    ///
    ///
    Run,

    ///
    ///
    None,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            state: State::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Update(text) => {
                self.state.value = text;
            }
            Msg::Clear => {
                self.state.entries.clear();
            }
            Msg::Up => todo!(),
            Msg::Down => todo!(),
            Msg::Run => {
                let cmd = self.state.value.trim();
                if !cmd.is_empty() {
                    match cmd {
                        "clear" => {
                            let _ = self.update(Msg::Clear);
                        }
                        _ => self.state.entries.push(self.state.value.to_owned()),
                    }
                }
                self.state.value = "".to_owned();
            }
            Msg::None => {}
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div id="outer-frame">
            <div id="inner-frame">
            { self.view_header() }
            <div class="container">
                { self.view_screen() }
                { self.view_input() }
            </div>
            { self.view_footer() }
            </div>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
