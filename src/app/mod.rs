// file: src/app/mod.rs
// authors: Brandon H. Gomes

//! Shell Application

pub mod command;
pub mod term;

/// Application exit status code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ExitStatus {
    /// Shell exited normally.
    Success,

    /// Shell did not exit normally.
    Failure,

    /// Shell did not start properly.
    FalseStart,

    /// Shell was unable to start due to an invalid initial environment.
    InvalidInitialEnvironment,

    /// Shell exited because of a failure during a terminal action.
    TerminalFailure,
}

impl ExitStatus {
    /// Convert an `ExitStatus` to a return code.
    pub fn report(self) -> i32 {
        self as i32
    }
}

/// Run shell application
pub fn run<Args>(args: Args) -> ExitStatus
where
    Args: Iterator<Item = String>,
{
    if let Ok(mut term) = term::setup(terminal::stdout()) {
        let exit_status = parse_args_and_run(&mut term, args);
        if term::exit(term).is_ok() {
            if let Ok(status) = exit_status {
                return status;
            }
        }
    }
    ExitStatus::TerminalFailure
}

fn parse_args_and_run<W, Args>(
    term: &mut terminal::Terminal<W>,
    mut args: Args,
) -> Result<ExitStatus, terminal::error::ErrorKind>
where
    W: std::io::Write,
    Args: Iterator<Item = String>,
{
    if let Some(arg) = args.next() {
        if arg == "--help" {
            command::help(term, "")?;
        }
        Ok(ExitStatus::Success)
    } else {
        run_inner(term)
    }
}

fn run_inner<W>(term: &mut terminal::Terminal<W>) -> Result<ExitStatus, terminal::error::ErrorKind>
where
    W: std::io::Write,
{
    let mut command_line = command::CommandLine::new();
    loop {
        command_line.draw(term)?;
        if let Some(signal) = term::next_key_event(&term).and_then(|k| command_line.update(k)) {
            match signal {
                command::CommandLineSignal::Control(c) => match c {
                    'd' => return Ok(ExitStatus::Success),
                    'c' => term::prefix_newline(term, "^C")?,
                    'l' => term::clear_and_reset_cursor(term)?,
                    _ => {}
                },
                command::CommandLineSignal::Enter { command, args } => {
                    term::newline(term)?;
                    let args = &args.to_owned();
                    match command {
                        "" => continue,
                        "exit" | "quit" => return Ok(ExitStatus::Success),
                        "clear" => command::clear(term, args)?,
                        "help" | "?" => command::help(term, args)?,
                        "history" => command::history(command_line.history(), term, args)?,
                        "hash" => command::hash(term, args)?,
                        _ => command::missing(command, term, args)?,
                    }
                }
            }
        }
    }
}
