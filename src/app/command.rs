// file: src/app/command.rs
// authors: Brandon H. Gomes

//! Shell Commmand Interface

use {
    super::term,
    std::io::Write,
    terminal::{self, Attribute, KeyCode, KeyEvent, KeyModifiers, Terminal},
};

pub(crate) type TerminalResult = terminal::error::Result<()>;

/// A container for the command line buffer and command history.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CommandLine {
    history: Vec<String>,
    index: usize,
    pointer: usize,

    /// Command prompt string.
    pub prompt: String,
}

/// Signal from [`CommandLine::update`]
///
/// [`CommandLine::update`]: struct.CommandLine.html#method.update
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CommandLineSignal<'e> {
    /// Control character
    Control(char),

    /// Entered command and arguments
    Enter { command: &'e str, args: &'e str },
}

impl CommandLine {
    /// Default prompt for the command line
    pub const DEFAULT_PROMPT: &'static str = "q❯ ";

    /// Build a command line from the given prompt string.
    pub fn with_prompt<S>(prompt: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            history: vec![String::default()],
            index: 0,
            pointer: 0,
            prompt: String::from(prompt.as_ref()),
        }
    }

    /// Build the default prompt.
    pub fn new() -> Self {
        Self::with_prompt(Self::DEFAULT_PROMPT)
    }

    /// Return a slice of the internal history buffer.
    pub fn history(&self) -> &[String] {
        &self.history[..self.history.len() - 2]
    }

    /// Clear history.
    pub fn clear(&mut self) {
        *self = Self::with_prompt(&self.prompt);
    }

    /// Run the `up` action on the command line.
    pub fn action_up(&mut self) {
        // TODO: fuzzy step through history
        if self.index > 0 {
            self.index -= 1;
            self.reset_pointer();
        }
    }

    /// Run the `down` action on the command line.
    pub fn action_down(&mut self) {
        // TODO: fuzzy step through history
        if self.index < self.history.len() - 1 {
            self.index += 1;
            self.reset_pointer();
        }
    }

    /// Run the `left` action on the command line.
    pub fn action_left(&mut self) {
        if self.pointer > 0 {
            self.pointer -= 1;
        }
    }

    /// Run the `right` action on the command line.
    pub fn action_right(&mut self) {
        if self.pointer < self.current().chars().count() {
            self.pointer += 1;
        }
    }

    /// Run the `tab` action on the command line.
    pub fn action_tab(&mut self) {
        // TODO: tab completion
    }

    /// Run the `^A` (control A) action on the command line.
    pub fn action_control_a(&mut self) {
        self.pointer = 0;
    }

    /// Run the `^E` (control E) action on the command line.
    pub fn action_control_e(&mut self) {
        self.reset_pointer()
    }

    /// Reset the history index.
    fn reset_index(&mut self) {
        self.index = self.history.len() - 1;
    }

    /// Reset the current line pointer.
    fn reset_pointer(&mut self) {
        self.pointer = self.current().chars().count();
    }

    /// Return shared reference to the [`current`] command line buffer.
    ///
    /// [`current`]: #method.current
    pub fn current(&self) -> &str {
        &self.history[self.index]
    }

    /// Insert a `char` to the [`current`] command line buffer.
    ///
    /// [`current`]: #method.current
    pub fn insert(&mut self, c: char) {
        let mut current = String::from(self.current());
        current.insert(self.pointer, c);
        self.pointer += 1;
        *self.history.last_mut().unwrap() = current;
        self.reset_index();
    }

    /// Remove a `char` from the [`current`] command line buffer.
    ///
    /// [`current`]: #method.current
    pub fn remove(&mut self) -> Option<char> {
        if self.pointer == 0 {
            return None;
        }
        let mut current = String::from(self.current());
        let result = current.remove(self.pointer - 1);
        self.pointer -= 1;
        *self.history.last_mut().unwrap() = current;
        self.reset_index();
        Some(result)
    }

    /// Reset the [`current`] buffer.
    ///
    /// [`current`]: #method.current
    pub fn reset_current(&mut self) {
        self.history.last_mut().unwrap().clear();
        self.reset_index();
        self.reset_pointer();
    }

    /// Save the [`current`] command line buffer to history, reset, and return parsed buffer.
    ///
    /// [`current`]: #method.current
    pub fn save_current(&mut self) -> (&str, &str) {
        // TODO: don't save to history if command begins with whitespace
        if self.current().is_empty() {
            return ("", "");
        }
        let len = self.history.len();
        if len > 1 {
            let current = self.current();
            if self.history[len - 2] == *current {
                self.history.last_mut().unwrap().clear();
                self.index = len - 1;
                self.reset_pointer();
                return Self::parse_buffer(&self.history[self.index - 1]);
            } else if self.index != len - 1 {
                *self.history.last_mut().unwrap() = current.to_owned();
            }
        }
        if !self.history.last().unwrap().is_empty() {
            self.history.push(String::default());
        }
        self.reset_index();
        self.reset_pointer();
        Self::parse_buffer(&self.history[self.index - 1])
    }

    /// Parse the given buffer by splitting along the first whitespace.
    fn parse_buffer(buffer: &str) -> (&str, &str) {
        let mut iter = buffer.trim().splitn(2, char::is_whitespace);
        match iter.next() {
            Some(command) => match iter.next() {
                Some(args) => (command, args.trim()),
                _ => (command, ""),
            },
            _ => ("", ""),
        }
    }

    /// Update the command line according to the given event.
    pub fn update(&mut self, key: KeyEvent) -> Option<CommandLineSignal> {
        match key.code {
            KeyCode::Up => self.action_up(),
            KeyCode::Down => self.action_down(),
            KeyCode::Left => self.action_left(),
            KeyCode::Right => self.action_right(),
            KeyCode::Tab => self.action_tab(),
            KeyCode::Backspace => {
                self.remove();
            }
            KeyCode::Char(c) => {
                if key.modifiers == KeyModifiers::CONTROL {
                    match c {
                        'c' | 'l' => self.reset_current(),
                        'a' => self.action_control_a(),
                        'e' => self.action_control_e(),
                        _ => {}
                    }
                    return Some(CommandLineSignal::Control(c));
                } else if key.modifiers.is_empty() {
                    self.insert(c);
                }
            }
            KeyCode::Enter => {
                let (command, args) = self.save_current();
                return Some(CommandLineSignal::Enter { command, args });
            }
            _ => {}
        }
        None
    }

    /// Draw the commmand line to the terminal.
    pub fn draw<W>(&self, term: &mut Terminal<W>) -> TerminalResult
    where
        W: Write,
    {
        // TODO: color command text if it is a valid command
        term::clear_line(term)?;
        term.write_all(self.prompt.as_bytes())?;
        term.write_all(self.current().as_bytes())?;
        term.flush()?;
        Ok(term::move_cursor_to(
            term,
            (self.prompt.chars().count() + self.pointer) as u16,
        )?)
    }
}

/// Report a missing command to the user.
pub fn missing<W>(command: &str, term: &mut Terminal<W>, args: &str) -> TerminalResult
where
    W: Write,
{
    // TODO: search through valid commands to see the closest fit
    term.write_fmt(format_args!(
        "[ERROR] missing command: {} {}",
        command, args
    ))?;
    term::newlines(term, 2)
}

/// Clear the terminal screen.
pub fn clear<W>(term: &mut Terminal<W>, args: &str) -> TerminalResult
where
    W: Write,
{
    if !args.is_empty() {
        term.write_all(b"usage: clear")?;
        term::newlines(term, 2)?;
    } else {
        term::clear_and_reset_cursor(term)?;
    }
    Ok(())
}

/// The command usage and help information.
pub(crate) const COMMAND_INFO: &[[&str; 3]] = &[
    ["clear", "Clear terminal screen", "usage: clear"],
    ["history", "Print command history", "usage: history"],
    ["hash", "Compute hash of <data>", "usage: hash <data>"],
];

/// Print information on available commands.
pub fn help<W>(term: &mut Terminal<W>, args: &str) -> TerminalResult
where
    W: Write,
{
    if args.is_empty() {
        term::newline(term)?;
        term::write_with_attribute(term, "qbar", Attribute::Bold)?;
        term.write_all(b" | ")?;
        term::write_with_attribute(term, "the rational proof assistant", Attribute::Italic)?;
        term::newlines(term, 2)?;
        term::prefix_newline(term, "Commands:")?;
        for entry in COMMAND_INFO {
            term.write_fmt(format_args!(
                "    {}{}{}",
                entry[0],
                " ".repeat(12 - core::cmp::min(12, entry[0].chars().count())),
                entry[1],
            ))?;
            term::newline(term)?;
        }
    } else {
        for entry in COMMAND_INFO {
            if args == entry[0] {
                term.write_all(entry[2].as_bytes())?;
                term::newlines(term, 2)?;
                return Ok(());
            }
        }
        term.write_fmt(format_args!("[ERROR] command not found: {}", args))?;
        term::newline(term)?;
    }
    term::newline(term)
}

/// Print the commmand history.
pub fn history<W>(history: &[String], term: &mut Terminal<W>, args: &str) -> TerminalResult
where
    W: Write,
{
    if !args.is_empty() {
        term.write_all(b"usage: history")?;
        term::newlines(term, 2)?;
    } else if !history.is_empty() {
        term.write_all(history.join(term::NEWLINE_STR).as_bytes())?;
        term::newlines(term, 2)?;
    }
    Ok(())
}

/// Compute hashes.
pub fn hash<W>(term: &mut Terminal<W>, args: &str) -> TerminalResult
where
    W: Write,
{
    // TODO: add custom algorithms
    use crate::hash::{algorithms::blake3, NamedHash};
    match blake3(args) {
        Ok(NamedHash { hash, .. }) => term.write_all(hash.as_bytes())?,
        Err(err) => term.write_fmt(format_args!(
            "[ERROR][{:?}] unable to hash with the blake3 algorithm",
            err
        ))?,
    }
    term::newlines(term, 2)
}
