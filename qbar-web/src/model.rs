// file: src/model.rs
// authors: Brandon H. Gomes

//! Application Model

use yew::prelude::*;

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
    started: bool,
}

impl State {
    ///
    ///
    pub fn new() -> Self {
        State {
            entries: Vec::new(),
            value: String::new(),
            started: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    ///
    ///
    pub fn view_input(&self) -> Html {
        html! {
            <input id="command-line"
                   autofocus=true
                   type="text"
                   name="command line"
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
        if self.state.started {
            html! {
                <div class="screen-wrapper flex-reverse">
                    <ul class="screen">
                        { for self.state.entries.iter().map(|e| self.view_entry(e)) }
                    </ul>
                </div>
            }
        } else {
            html! {
                <div class="screen-wrapper">
                    { self.view_welcome_screen() }
                </div>
            }
        }
    }

    ///
    ///
    pub fn view_welcome_screen(&self) -> Html {
        html! {
            <div class="welcome-screen">
                <h1 class="welcome-title">
                    <b>{"qbar"}</b>{" | "}<i>{"the rational proof assistant"}</i>
                </h1>
                <p>
                    {"The following is a demo version of the rational proof assistant. See the "}
                    <a href="https://qbar.io/install">{"install page"}</a>
                    {" for more information on the latest version of the full command line application."}
                </p>
                <br/>
                <p>{"type "}<b><code>{"?"}</code></b>{" for help information"}</p>
            </div>
        }
    }

    ///
    ///
    pub fn view_entry(&self, entry: &str) -> Html {
        html! {
            <li><pre class="entry">{entry}</pre></li>
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
                        "help" | "?" => {
                            self.state.entries.push("help information ...".to_owned());
                        }
                        _ => {
                            self.state
                                .entries
                                .push(format!("missing command: {}", self.state.value));
                        }
                    }
                    self.state.started = true;
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
