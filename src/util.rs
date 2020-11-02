// file: src/util.rs
// authors: Brandon H. Gomes

//! Utilities Module

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
