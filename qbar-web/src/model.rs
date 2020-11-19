// file: src/model.rs
// authors: Brandon H. Gomes

//! Application Model

use {
    crate::term::Terminal,
    qbar_core::term::{Term, TermDone, Writer},
    yew::prelude::*,
};

/// Application Model
#[derive(Clone, Debug)]
pub struct Model {
    link: ComponentLink<Self>,
    terminal: Terminal,
    value: String,
    started: bool,
}

impl Model {
    /// Get the command line input view.
    pub fn view_input(&self) -> Html {
        html! {
            <input id="command-line"
                   autofocus=true
                   type="text"
                   name="command line"
                   value=&self.value
                   oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                   onkeypress=self.link.callback(|e: KeyboardEvent| {
                       if e.key() == "Enter" { Msg::Run } else { Msg::None }
                   }) />
        }
    }

    /// Get the terminal screen view.
    pub fn view_screen(&self) -> Html {
        if self.started {
            html! {
                <div class="screen-wrapper flex-reverse">
                    { self.terminal.get_html() }
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

    /// Get the welcome screen view.
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

    /// Get the header view.
    pub fn view_header(&self) -> Html {
        // TODO: pass this to static html
        html! {
            <header>
                <a target="_blank" rel="noreferrer" href="https://qbar.io">
                    <img id="logo" type="image/svg" alt="qbar logo" src="../image/icon_trimmed.svg"/>
                </a>
            </header>
        }
    }

    /// Get the footer view.
    pub fn view_footer(&self) -> Html {
        // TODO: pass this to static html
        html! {
            <footer>
                {"2020 © "}
                <a class="footer-link" target="_blank" rel="noreferrer" href="https://bhgomes.dev">
                    {"Brandon H. Gomes"}
                </a>
            </footer>
        }
    }

    /// Clear terminal.
    pub fn clear(&mut self) {
        let _ = self.terminal.clear_all();
    }

    /// Run the current active command.
    pub fn run(&mut self) -> TermDone<Terminal> {
        let cmd = self.value.trim();
        if !cmd.is_empty() {
            match cmd {
                "clear" => self.clear(),
                "help" | "?" => {
                    let _ = self.terminal.log_info("help information ...")?.done();
                }
                _ => {
                    let _ = self
                        .terminal
                        .log_err(format!("missing command: {}", self.value))?
                        .done();
                }
            }
            self.started = true;
        }
        self.value = "".to_owned();
        Ok(())
    }
}

/// User Message
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Msg {
    /// Update the command line
    Update(String),

    /// Clear the screen
    Clear,

    /// Move up in the command history
    Up,

    /// Move down in the command history
    Down,

    /// Run the current active command
    Run,

    /// Do nothing
    None,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();

    #[inline]
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            terminal: Default::default(),
            value: Default::default(),
            started: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Update(text) => {
                self.value = text;
            }
            Msg::Clear => self.clear(),
            Msg::Up => todo!(),
            Msg::Down => todo!(),
            Msg::Run => {
                let _ = self.run();
            }
            Msg::None => {}
        }
        true
    }

    #[inline]
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        // TODO: how much of this can be put in static html?
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
