// file: src/parser.rs
// authors: Brandon H. Gomes

//! Expression Parsing Module

use rational_deduction::*;

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
