// file: src/lib.rs
// authors: Brandon H. Gomes

//! The Rational Proof Assistant

use {core::borrow::Borrow, rational_deduction::*};

pub mod app;

/// Parsed Expression
pub trait ParsedExpression
where
    Self: Expression,
{
    /// Parse an `Expression` using a given classification scheme
    fn parse<T, C, I>(classify: C, iter: I) -> parse::Result<Self>
    where
        Self::Atom: Default + Extend<T>,
        C: Fn(&T) -> parse::SymbolType,
        I: IntoIterator<Item = T>,
    {
        parse::parse(classify, iter)
    }
}

/// Expression Parsing Module
pub mod parse {
    use {
        super::*,
        core::{iter::Peekable, result},
    };

    /// Expression Parsing Error
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Error {
        /// Multiple expressions at top level
        MultiExpr,

        /// No closing quote
        MissingQuote,

        /// Group was not closed
        OpenGroup,

        /// Group was not opened
        UnopenedGroup,
    }

    /// Expression Parsing Result Alias
    pub type Result<T> = result::Result<T, Error>;

    /// Meaningful symbols for parsing algorithm
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum SymbolType {
        /// Whitespace
        Whitespace,

        /// Start of a group
        GroupOpen,

        /// End of a group
        GroupClose,

        /// Start of a quoted sub-string
        Quote,

        /// Other characters
        Other,
    }

    impl SymbolType {
        /// Checks if the classified symbol is whitespace
        pub fn is_whitespace<T, C>(classify: C) -> impl Fn(&T) -> bool
        where
            C: Fn(&T) -> SymbolType,
        {
            move |t| classify(t) == SymbolType::Whitespace
        }

        /// Checks if the classified symbol is not whitespace
        pub fn is_not_whitespace<T, C>(classify: C) -> impl Fn(&T) -> bool
        where
            C: Fn(&T) -> SymbolType,
        {
            move |t| classify(t) != SymbolType::Whitespace
        }
    }

    pub(crate) fn parse<T, E, C, I>(classify: C, iter: I) -> Result<E>
    where
        E: Expression,
        E::Atom: Default + Extend<T>,
        C: Fn(&T) -> SymbolType,
        I: IntoIterator<Item = T>,
    {
        let mut stripped = iter
            .into_iter()
            .skip_while(SymbolType::is_whitespace(&classify))
            .peekable();
        match stripped.next() {
            Some(first) => match classify(&first) {
                SymbolType::GroupOpen => parse_group(&classify, stripped),
                SymbolType::GroupClose => Err(Error::UnopenedGroup),
                _ => {
                    let atom =
                        parse_atom_without_leading_whitespace(&classify, first, &mut stripped)?;
                    if let Some(next) = stripped.next() {
                        match classify(&next) {
                            SymbolType::Whitespace => {
                                if stripped.any(|t| SymbolType::is_not_whitespace(&classify)(&t)) {
                                    return Err(Error::MultiExpr);
                                }
                            }
                            SymbolType::GroupOpen | SymbolType::GroupClose => {
                                return Err(Error::MultiExpr);
                            }
                            _ => {}
                        }
                    }
                    Ok(atom)
                }
            },
            _ => Ok(E::from_atom(Default::default())),
        }
    }

    pub(crate) fn parse_group<T, E, C, I>(classify: C, mut iter: Peekable<I>) -> Result<E>
    where
        E: Expression,
        E::Atom: Default + Extend<T>,
        C: Fn(&T) -> SymbolType,
        I: Iterator<Item = T>,
    {
        let mut groups = Vec::default();
        groups.push(E::Group::default());
        while let Some(next) = iter.next() {
            match classify(&next) {
                SymbolType::Whitespace => continue,
                SymbolType::GroupOpen => groups.push(E::Group::default()),
                SymbolType::GroupClose => {
                    let last = E::from_group(groups.pop().unwrap());
                    if groups.is_empty() {
                        return if iter.find(SymbolType::is_not_whitespace(classify)).is_none() {
                            Ok(last)
                        } else {
                            Err(Error::MultiExpr)
                        };
                    } else {
                        groups.last_mut().unwrap().extend(Some(last));
                    }
                }
                _ => {
                    groups
                        .last_mut()
                        .unwrap()
                        .extend(Some(parse_atom_without_leading_whitespace(
                            &classify, next, &mut iter,
                        )?))
                }
            }
        }
        Err(Error::OpenGroup)
    }

    pub(crate) fn parse_atom_without_leading_whitespace<T, E, C, I>(
        classify: C,
        head: T,
        tail: &mut Peekable<I>,
    ) -> Result<E>
    where
        E: Expression,
        E::Atom: Default + Extend<T>,
        C: Fn(&T) -> SymbolType,
        I: Iterator<Item = T>,
    {
        let mut atom = E::Atom::default();
        match classify(&head) {
            SymbolType::Quote => {
                if !parse_quoted_atom(&classify, head, tail, &mut atom) {
                    return Err(Error::MissingQuote);
                }
            }
            _ => atom.extend(Some(head)),
        }
        while let Some(peek) = tail.peek() {
            match classify(&peek) {
                SymbolType::Whitespace | SymbolType::GroupOpen | SymbolType::GroupClose => {
                    break;
                }
                SymbolType::Quote => {
                    if !parse_quoted_atom(&classify, tail.next().unwrap(), tail, &mut atom) {
                        return Err(Error::MissingQuote);
                    }
                }
                _ => atom.extend(tail.next()),
            }
        }
        Ok(E::from_atom(atom))
    }

    pub(crate) fn parse_quoted_atom<T, A, C, I>(
        classify: C,
        head: T,
        tail: &mut I,
        atom: &mut A,
    ) -> bool
    where
        A: Default + Extend<T>,
        C: Fn(&T) -> SymbolType,
        I: Iterator<Item = T>,
    {
        atom.extend(Some(head));
        tail.any(move |a| {
            let symbol_type = classify(&a);
            atom.extend(Some(a));
            symbol_type == SymbolType::Quote
        })
    }
}

/// Database key which generates a default value.
pub trait DatabaseKey<V> {
    /// Generate a key from the value.
    fn from_value(value: &V) -> Self;
}

/// Database entry which represents a key value pair with possibly generated keys.
pub trait DatabaseEntry<K, V>
where
    K: DatabaseKey<V>,
{
    /// Get the database key.
    fn key(&self) -> &K;

    /// Get the database value.
    fn value(&self) -> &V;

    /// Generate a new entry from a key and a value.
    fn new(key: K, value: V) -> Self;

    /// Generate a new entry from a value using the generated key.
    fn from_value(value: V) -> Self
    where
        Self: Sized,
    {
        Self::new(K::from_value(&value), value)
    }
}

/// Database Trait
pub trait Database<K, V>
where
    K: DatabaseKey<V>,
{
    /// Returns a reference to the value corresponding to the key.
    fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        Q: ?Sized + Borrow<K>;

    /// Returns a mutable reference to the value corresponding to the key.
    fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        Q: ?Sized + Borrow<K>;

    /// Returns `true` if the database contains a value for the specified key.
    fn contains_key<Q>(&self, k: &Q) -> bool
    where
        Q: ?Sized + Borrow<K>;

    /// Returns `true` if the database contains the value.
    fn contains_value(&self, v: &V) -> bool {
        self.contains_key(&K::from_value(v))
    }

    /// Inserts a key-value pair into the database.
    fn insert(&mut self, k: K, v: V) -> Option<K>;

    /// Inserts a value into the database using the generated key.
    fn insert_value(&mut self, v: V) -> Option<K>;

    /// Removes a key from the database, returning the value at the key if the
    /// key was previously in the map.
    fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        Q: ?Sized + Borrow<K>;

    /// Removes a value from the database using the generated key, returning
    /// the value if the value was previously in the database.
    fn remove_value(&mut self, v: &V) -> Option<V> {
        self.remove(&K::from_value(v))
    }

    /// Retains only the elements specified by the predicate.
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool;

    /// Get the length of the database.
    fn len(&self) -> usize;

    /// Check if the database is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the database.
    fn clear(&mut self);
}

/// Expression Database Utilities
pub mod database {}

/// Hashing Utilities
pub mod hash {
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
}

/// Utilities
pub mod util {
    use core::fmt;

    /// Format an iterator pointwise with a spacer.
    pub fn format_iter_with_spacer<T, I, FT, FS>(
        iter: I,
        f: &mut fmt::Formatter,
        format: FT,
        format_spacer: FS,
    ) -> fmt::Result
    where
        I: IntoIterator<Item = T>,
        FT: Fn(T, &mut fmt::Formatter) -> fmt::Result,
        FS: Fn(&mut fmt::Formatter) -> fmt::Result,
    {
        let mut iter = iter.into_iter();
        iter.next()
            .map(move |n| {
                format(n, f).and(iter.try_fold((), move |_, t| format_spacer(f).and(format(t, f))))
            })
            .unwrap_or(Ok(()))
    }
}
