// file: src/term.rs
// authors: Brandon H. Gomes

//! Web Terminal Implementation

use {
    core::fmt,
    qbar::{
        command::CommandHistory,
        term::{
            Attribute, Clear, Color, Event, Term, TermContinue, TermDone, TermReturn, WriteStyle,
            Writer, WriterContinue, WriterDone,
        },
    },
    std::collections::VecDeque,
    yew::{html, Html},
};

/// Convert a string to a `<pre>` element.
pub fn preformatted(style: &str, text: &str) -> Html {
    if style.trim().is_empty() {
        html! { <>{text}</> }
    } else {
        html! { <span style=style>{text}</span> }
    }
}

/// Color to RGB style text.
pub fn color_to_rgb_text(color: Color) -> String {
    format!("rgb({}, {}, {})", color.0, color.1, color.2)
}

/// Color as a foreground style.
pub fn as_foreground(color: Color) -> String {
    format!("color: {}", color_to_rgb_text(color))
}

/// Color as a background style.
pub fn as_background(color: Color) -> String {
    format!("background: {}", color_to_rgb_text(color))
}

/// Color pair as a foreground/background style.
pub fn as_color_style(foreground: Color, background: Color) -> String {
    format!(
        "color: {}; background: {}",
        color_to_rgb_text(foreground),
        color_to_rgb_text(background)
    )
}

/// Add attribute to HTML.
pub fn add_attribute(attr: Attribute, html: Html) -> Html {
    match attr {
        Attribute::Bold => html! { <b>{html}</b> },
        Attribute::Italic => html! { <i>{html}</i> },
        Attribute::BoldItalic => {
            add_attribute(Attribute::Bold, add_attribute(Attribute::Italic, html))
        }
        Attribute::Underlined => html,
        Attribute::BoldUnderlined => {
            add_attribute(Attribute::Bold, add_attribute(Attribute::Underlined, html))
        }
        Attribute::ItalicUnderlined => add_attribute(
            Attribute::Italic,
            add_attribute(Attribute::Underlined, html),
        ),
        Attribute::BoldItalicUnderlined => add_attribute(
            Attribute::BoldItalic,
            add_attribute(Attribute::Underlined, html),
        ),
        _ => html,
    }
}

/// Convert styled text to HTML.
pub fn styled_text_to_html(text: &str, style: WriteStyle) -> Html {
    match style {
        WriteStyle::None => preformatted("", text),
        WriteStyle::Foreground(color) => preformatted(&as_foreground(color), text),
        WriteStyle::Background(color) => preformatted(&as_background(color), text),
        WriteStyle::Color(foreground, background) => {
            preformatted(&as_color_style(foreground, background), text)
        }
        WriteStyle::Attribute(attr) => {
            add_attribute(attr, styled_text_to_html(text, WriteStyle::None))
        }
        WriteStyle::ForegroundAttribute(color, attr) => add_attribute(
            attr,
            styled_text_to_html(text, WriteStyle::Foreground(color)),
        ),
        WriteStyle::BackgroundAttribute(color, attr) => add_attribute(
            attr,
            styled_text_to_html(text, WriteStyle::Background(color)),
        ),
        WriteStyle::ColorAttribute(foreground, background, attr) => add_attribute(
            attr,
            styled_text_to_html(text, WriteStyle::Color(foreground, background)),
        ),
    }
}

/// Web Terminal
#[derive(Clone, Debug, PartialEq)]
pub struct Terminal {
    history: CommandHistory,
    entries: VecDeque<Html>,
    max_capacity: usize,
    drop_count: usize,
    is_flushed: bool,
}

impl Terminal {
    /// Build a new Terminal with the given `max_capacity` and `drop_count`.
    ///
    /// Returns `None` if the `drop_count == 0` or if `drop_count >= max_capacity`.
    pub fn new(max_capacity: usize, drop_count: usize) -> Option<Self> {
        if drop_count >= 1 && drop_count < max_capacity {
            Some(Self {
                history: CommandHistory::new(),
                entries: VecDeque::with_capacity(drop_count),
                max_capacity,
                drop_count,
                is_flushed: false,
            })
        } else {
            None
        }
    }

    /// Return the maximum capacity of the internal buffer.
    #[inline]
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    /// Return the number of elements to be dropped on every resize.
    #[inline]
    pub fn drop_count(&self) -> usize {
        self.drop_count
    }

    /// Drop elements if the length of the internal buffer equals the capacity.
    pub fn maybe_drop(&mut self) -> bool {
        if self.entries.len() == self.max_capacity {
            self.entries.drain(0..self.drop_count - 1).last();
            true
        } else {
            false
        }
    }

    fn entry_html(html: Html) -> Html {
        html! { <li><pre class="entry">{html}</pre></li> }
    }

    /// Get an HTML representation of the `Terminal`.
    #[inline]
    pub fn get_html(&self) -> Html {
        // TODO: can we do this without cloning?
        html! { <ul class="screen">{for self.entries.iter().cloned().map(Self::entry_html)}</ul> }
    }
}

impl Default for Terminal {
    #[inline]
    fn default() -> Self {
        Self::new(1024, 256).unwrap()
    }
}

impl Writer for Terminal {
    type Error = ();

    fn write_styled<D>(&mut self, display: D, style: WriteStyle) -> WriterContinue<Self>
    where
        D: fmt::Display,
    {
        let entry = styled_text_to_html(&display.to_string(), style);
        if self.is_flushed || self.entries.is_empty() {
            self.is_flushed = false;
            self.maybe_drop();
            self.entries.push_back(entry);
        } else {
            let last = self.entries.pop_back().unwrap();
            self.entries.push_back(html! { <>{last}{entry}</> });
        }
        Ok(self)
    }

    #[inline]
    fn done(&mut self) -> WriterDone<Self> {
        self.is_flushed = true;
        Ok(())
    }
}

impl Term for Terminal {
    #[inline]
    fn start(self) -> TermReturn<Self, Self> {
        Ok(self)
    }

    #[inline]
    fn stop(self) -> TermDone<Self> {
        Ok(())
    }

    fn next(&mut self) -> Option<Event> {
        todo!()
    }

    fn size(&self) -> TermReturn<Self, (u16, u16)> {
        todo!()
    }

    #[inline]
    fn to_column(&mut self, column: u16) -> TermContinue<Self> {
        let _ = column;
        Ok(self)
    }

    fn clear(&mut self, clear: Clear) -> TermContinue<Self> {
        match clear {
            Clear::CurrentLine => {
                // TODO: clear the current command history line
                // `self.history.current_mut().clear()` like this?
            }
            Clear::All => self.entries.clear(),
        }
        Ok(self)
    }
}
