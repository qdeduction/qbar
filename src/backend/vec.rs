// file: src/backend/vec.rs
// authors: Brandon H. Gomes

//! Default implementations using `std::vec::Vec`

use std::vec;

/// `Vec`-based `Expression`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub enum VecExpr<A> {
    /// An atomic element of type `A`
    Atom(A),

    /// A grouped vector expression
    Group(Vec<Self>),
}

impl<A> Expression for VecExpr<A> {
    type Atom = A;

    type Group = Vec<Self>;

    type GroupIter = vec::IntoIter<Self>;

    /// Convert `VecExpr` enum to default enum implementation.
    fn into_expr(self) -> Expr<Self> {
        match self {
            VecExpr::Atom(atom) => Expr::Atom(atom),
            VecExpr::Group(group) => Expr::Group(group),
        }
    }

    /// Apply `Atom` constructor.
    fn from_atom(atom: A) -> Self {
        VecExpr::Atom(atom)
    }

    /// Apply `Group` constructor.
    fn from_group(group: Vec<Self>) -> Self {
        VecExpr::Group(group)
    }
}

impl<A> ParsedExpression for VecExpr<A> {}
