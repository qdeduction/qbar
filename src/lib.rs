// file: src/lib.rs
// authors: Brandon H. Gomes

//! The Rational Proof Assistant

use core::iter::FromIterator;

/// Expression Tree
pub trait Expression
where
    Self: Sized,
{
    /// Atomic element type
    type Atom;

    /// Group expression type
    type Group: Default
        + Extend<Self>
        + IntoIterator<Item = Self, IntoIter = Self::GroupIter>
        + FromIterator<Self>;

    /// Iterator type to read from [`Group`]
    ///
    /// [`Group`]: #associatedtype.Group
    type GroupIter: Iterator<Item = Self>;

    /// Convert to [canonical enumeration]
    ///
    /// [canonical enumeration]: enum.Expr.html
    fn into_expr(self) -> Expr<Self>;

    /// Build an `Expression` from an atomic element.
    fn from_atom(atom: Self::Atom) -> Self;

    /// Build an `Expression` from a grouped expression.
    fn from_group(group: Self::Group) -> Self;

    /// Convert from [canonical enumeration]
    ///
    /// [canonical enumeration]: enum.Expr.html
    fn from_expr(expr: Expr<Self>) -> Self {
        match expr {
            Expr::Atom(atom) => Self::from_atom(atom),
            Expr::Group(group) => Self::from_group(group),
        }
    }

    /// Get default `Expression` from canonical enumeration
    fn default() -> Self {
        Self::from_expr(Default::default())
    }
}

/// Canonical Concrete Expression Type
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Expr<E>
where
    E: Expression,
{
    /// Atomic element
    Atom(E::Atom),

    /// Grouped expression
    Group(E::Group),
}

impl<E> Expr<E>
where
    E: Expression,
{
    /// Check if expression is an atomic expression
    #[must_use]
    pub fn is_atom(&self) -> bool {
        match self {
            Expr::Atom(_) => true,
            _ => false,
        }
    }

    /// Check if expression is a grouped expression
    #[must_use]
    pub fn is_group(&self) -> bool {
        match self {
            Expr::Group(_) => true,
            _ => false,
        }
    }
}

impl<E> Default for Expr<E>
where
    E: Expression,
{
    /// Default Expr
    ///
    /// The default expression is the empty group expression.
    fn default() -> Self {
        Self::Group(E::Group::default())
    }
}

impl<E> Expression for Expr<E>
where
    E: Expression,
{
    type Atom = E::Atom;

    type Group = expr::ExprGroup<E>;

    type GroupIter = expr::ExprGroupIter<E>;

    fn into_expr(self) -> Expr<Self> {
        match self {
            Expr::Atom(atom) => Expr::Atom(atom),
            Expr::Group(group) => Expr::Group(expr::ExprGroup { group }),
        }
    }

    fn from_atom(atom: <Self as Expression>::Atom) -> Self {
        Self::Atom(atom)
    }

    fn from_group(group: <Self as Expression>::Group) -> Self {
        Self::Group(group.group)
    }
}

/// Parsed Expression
pub trait ParsedExpression
where
    Self: Expression,
{
    /// Parse an `Expression` using a given classification scheme
    fn parse<T, C, I>(classify: C, iter: I) -> expr::parse::Result<Self>
    where
        Self::Atom: Default + Extend<T>,
        C: Fn(&T) -> expr::parse::SymbolType,
        I: IntoIterator<Item = T>;
}

/// Expression Module
pub mod expr {
    use {
        super::{Expr, Expression},
        core::iter::FromIterator,
    };

    /// Expression Group Container
    pub struct ExprGroup<E>
    where
        E: Expression,
    {
        pub group: E::Group,
    }

    impl<E> Default for ExprGroup<E>
    where
        E: Expression,
    {
        fn default() -> Self {
            ExprGroup {
                group: E::Group::default(),
            }
        }
    }

    impl<E> Extend<Expr<E>> for ExprGroup<E>
    where
        E: Expression,
    {
        fn extend<I>(&mut self, iter: I)
        where
            I: IntoIterator<Item = Expr<E>>,
        {
            self.group.extend(iter.into_iter().map(E::from_expr))
        }
    }

    impl<E> IntoIterator for ExprGroup<E>
    where
        E: Expression,
    {
        type Item = Expr<E>;

        type IntoIter = ExprGroupIter<E>;

        fn into_iter(self) -> Self::IntoIter {
            ExprGroupIter {
                iter: self.group.into_iter(),
            }
        }
    }

    impl<E> FromIterator<Expr<E>> for ExprGroup<E>
    where
        E: Expression,
    {
        fn from_iter<I>(iter: I) -> Self
        where
            I: IntoIterator<Item = Expr<E>>,
        {
            Self {
                group: E::Group::from_iter(iter.into_iter().map(E::from_expr)),
            }
        }
    }

    /// Expression Group Container Iterator
    pub struct ExprGroupIter<E>
    where
        E: Expression,
    {
        iter: E::GroupIter,
    }

    impl<E> Iterator for ExprGroupIter<E>
    where
        E: Expression,
    {
        type Item = Expr<E>;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next().map(E::into_expr)
        }
    }

    /// Expression Parsing Module
    pub mod parse {
        use core::result;

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
