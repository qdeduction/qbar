// file: src/term.rs
// authors: Brandon H. Gomes

//! Terminal Utilities Module

use {
    std::io::Write,
    terminal::{
        error::Result, Action, Attribute, Clear, Event, KeyCode, KeyEvent, KeyModifiers, Retrieved,
        Terminal, Value,
    },
};

/// Canonical New Line String
pub const NEWLINE_STR: &str = "\r\n";

/// Setup the application terminal and exit if setup fails.
pub fn setup<W>(term: Terminal<W>) -> Result<Terminal<W>>
where
    W: Write,
{
    match setup_inner(&term) {
        Ok(_) => Ok(term),
        Err(err) => exit(term).and(Err(err)),
    }
}

/// Setup the application terminal.
fn setup_inner<W>(term: &Terminal<W>) -> Result<()>
where
    W: Write,
{
    term.act(Action::EnableRawMode)
}

/// Tear down the application terminal.
pub fn exit<W>(term: Terminal<W>) -> Result<()>
where
    W: Write,
{
    term.act(Action::DisableRawMode)?;
    term.act(Action::EnterAlternateScreen)?;
    Ok(())
}

/// Clear the entire terminal.
pub fn clear<W>(term: &Terminal<W>) -> Result<()>
where
    W: Write,
{
    term.act(Action::ClearTerminal(Clear::All))
}

/// Clear the current line.
pub fn clear_line<W>(term: &mut Terminal<W>) -> Result<()>
where
    W: Write,
{
    term.act(Action::ClearTerminal(Clear::CurrentLine))?;
    term.write_all(b"\r")?;
    Ok(term.flush()?)
}

/// Reset the cursor.
pub fn reset_cursor<W>(term: &Terminal<W>) -> Result<()>
where
    W: Write,
{
    term.act(Action::MoveCursorTo(0, 0))
}

/// Clear the entire terminal and reset the cursor.
pub fn clear_and_reset_cursor<W>(term: &mut Terminal<W>) -> Result<()>
where
    W: Write,
{
    clear(&term).and(reset_cursor(&term))
}

/// Move cursor horizontally to the given column.
pub fn move_cursor_to<W>(term: &Terminal<W>, column: u16) -> Result<()>
where
    W: Write,
{
    if let Retrieved::CursorPosition(_, row) = term.get(Value::CursorPosition)? {
        term.act(Action::MoveCursorTo(column, row))
    } else {
        Err(terminal::error::ErrorKind::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "failed to retrieve cursor position",
        )))
    }
}

/// Write a newline to the terminal.
pub fn newline<W>(term: &mut Terminal<W>) -> Result<()>
where
    W: Write,
{
    Ok(term.write_all(NEWLINE_STR.as_bytes()).and(term.flush())?)
}

/// Write a newline to the terminal after a given prefix.
pub fn prefix_newline<W, S>(term: &mut Terminal<W>, prefix: S) -> Result<()>
where
    W: Write,
    S: AsRef<str>,
{
    term.write_all(prefix.as_ref().as_bytes())?;
    newline(term)
}

/// Write newlines to the terminal.
pub fn newlines<W>(term: &mut Terminal<W>, count: usize) -> Result<()>
where
    W: Write,
{
    if count > 0 {
        prefix_newline(term, "\n".repeat(count - 1))
    } else {
        Ok(())
    }
}

/// Write text to terminal with attribute enabled.
pub fn write_with_attribute<W, S>(
    term: &mut Terminal<W>,
    text: S,
    attribute: Attribute,
) -> Result<()>
where
    W: Write,
    S: AsRef<str>,
{
    term.act(Action::SetAttribute(attribute))?;
    term.write_all(text.as_ref().as_bytes())?;
    term.act(Action::SetAttribute(Attribute::Reset))?;
    Ok(term.flush()?)
}

/// Try to get the next terminal event.
pub fn next_event<W>(term: &Terminal<W>) -> Option<Event>
where
    W: Write,
{
    match term.get(Value::Event(None)) {
        Ok(Retrieved::Event(event)) => event,
        _ => None,
    }
}

/// Try to get the next terminal key event.
pub fn next_key_event<W>(term: &Terminal<W>) -> Option<KeyEvent>
where
    W: Write,
{
    match next_event(term) {
        Some(Event::Key(key)) => Some(key),
        _ => None,
    }
}

/// State of a yes/no prompt.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum YNPromptState {
    /// Positive state
    Yes,

    /// Negative state
    No,

    /// Prompt exit state
    ControlC,

    /// Shell exit state
    ControlD,
}

impl From<bool> for YNPromptState {
    fn from(b: bool) -> Self {
        if b {
            Self::Yes
        } else {
            Self::No
        }
    }
}

/// Start a [`YNPromptState`] loop.
///
/// [`YNPromptState`]: enum.YNPromptState.html
pub fn yn_prompt<W>(
    term: &mut Terminal<W>,
    prefix: &[u8],
    default: bool,
) -> terminal::error::Result<YNPromptState>
where
    W: Write,
{
    let mut buffer = String::with_capacity(3);
    loop {
        clear_line(term)?;
        term.write_all(prefix)?;
        term.write_all(buffer.as_bytes())?;
        term.flush()?;
        if let Some(key) = next_key_event(term) {
            match key.code {
                KeyCode::Backspace => {
                    buffer.pop();
                }
                KeyCode::Char(c) => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        match c {
                            'c' => return Ok(YNPromptState::ControlC),
                            'd' => return Ok(YNPromptState::ControlD),
                            _ => {}
                        }
                    } else {
                        buffer.push(c);
                    }
                }
                KeyCode::Enter => {
                    if buffer.is_empty() {
                        return Ok(default.into());
                    } else {
                        match buffer.to_ascii_lowercase().as_str() {
                            "y" | "yes" => return Ok(YNPromptState::Yes),
                            "n" | "no" => return Ok(YNPromptState::No),
                            _ => buffer.clear(),
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
