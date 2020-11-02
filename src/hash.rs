// file: src/hash.rs
// authors: Brandon H. Gomes

//! Hashing Utilities

use std::{
    io::{self, Write},
    process,
};

/// Hash with traced source
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NamedHash<H, S> {
    /// Hash value
    pub hash: H,

    /// Hash source
    pub source: S,
}

/// Source of hash which results from a process.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HashProcessSource<'s> {
    /// Hash command
    pub command: &'s str,

    /// Command arguments
    pub args: &'s [&'s str],
}

/// Hash which results from a process.
pub type HashFromProcess<'s> = NamedHash<String, HashProcessSource<'s>>;

/// Compute hash from a process.
pub fn from_process<'s, S>(
    command: &'s str,
    args: &'s [&str],
    input: S,
) -> io::Result<HashFromProcess<'s>>
where
    S: AsRef<str>,
{
    let mut child = process::Command::new(command)
        .args(args)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()?;
    match child.stdin.as_mut() {
        Some(stdin) => stdin.write_all(input.as_ref().as_bytes())?,
        _ => return Err(io::Error::new(io::ErrorKind::Other, "unable to open stdin")),
    }
    Ok(NamedHash {
        hash: String::from(String::from_utf8_lossy(&child.wait_with_output()?.stdout).trim()),
        source: HashProcessSource { command, args },
    })
}

/// Compute hash from `HashProcessSource`.
pub fn from_process_source<'s, S>(
    source: &HashProcessSource<'s>,
    input: S,
) -> io::Result<HashFromProcess<'s>>
where
    S: AsRef<str>,
{
    from_process(source.command, source.args, input)
}

/// Implemented Hashing Algorithms
pub mod algorithms {
    use super::*;

    /// Blake3 Hashing Parameters
    pub const BLAKE3_HASH_SOURCE: HashProcessSource<'static> = HashProcessSource {
        command: "b3sum",
        args: &["--no-names"],
    };

    /// Blake3 Hashing Algorithm
    pub fn blake3<S>(input: S) -> io::Result<HashFromProcess<'static>>
    where
        S: AsRef<str>,
    {
        from_process_source(&BLAKE3_HASH_SOURCE, input)
    }
}
