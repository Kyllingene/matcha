//! A framework for breaking down token streams in proc macros.
//!
//! The macros [`decompose!`] and [`compose!`] are the main entrypoints for this crate. However,
//! it's worth familiarizing yourself with all the types/traits.
//!
//! # Features
//!
//! The following features are provided (all are enabled by default).
//!
//! - `quote`: enables [`quote::ToTokens`] impls for various types.
//! - `proc-macro2`: uses the [`proc-macro2`](proc-macro2) crate instead of the
//!   built-in `proc-macro`.
//! - `proc-macro2-span-locations`: enables the `proc-macro2` feature
//!   `span-locations`, allowing [errors](Error) to print their locations.
#![cfg_attr(not(feature = "quote"), expect(rustdoc::broken_intra_doc_links))]

#[cfg(not(feature = "proc-macro2"))]
extern crate proc_macro;

#[cfg(feature = "proc-macro2")]
#[doc(hidden)]
pub use proc_macro2 as procmacro;

#[cfg(not(feature = "proc-macro2"))]
#[doc(hidden)]
pub use proc_macro as procmacro;

mod compose;
mod decompose;
mod error;
mod impls;
mod stream;
mod utils;

pub use error::{Error, ErrorText};
pub use stream::{DiagDisplay, FromStream, MatchStream, Stream, StreamLike, StreamView};
pub use utils::{
    Braces, Brackets, Cut, Delimited, Greedy, Group, GroupKind, Ident, Literal, Maybe, Neg, Parens,
    Punct, punct,
};

/// Generates a new compile error pointing at the given span.
///
/// # Examples
///
/// ```
/// # use std::str::FromStr;
/// # use proc_macro2::TokenStream;
/// # fn foo() -> TokenStream {
/// # let mut stream = proc_macro2::TokenStream::from_str("123").unwrap().into_iter();
/// if let Some(tt) = stream.next() {
///     return matcha::error(format!("expected end of file, found `{}`", tt.to_string()), tt.span());
/// }
/// # TokenStream::new()
/// # }
/// ```
pub fn error(msg: impl AsRef<str>, span: procmacro::Span) -> procmacro::TokenStream {
    use procmacro::{Delimiter, Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};

    let mut ident = Ident::new("compile_error", span);
    ident.set_span(span);

    let mut bang = Punct::new('!', Spacing::Joint);
    bang.set_span(span);

    let mut msg = Literal::string(msg.as_ref());
    msg.set_span(span);

    let mut group_stream = TokenStream::new();
    group_stream.extend([TokenTree::Literal(msg)]);
    let mut group = Group::new(Delimiter::Parenthesis, group_stream);
    group.set_span(span);

    let mut semicolon = Punct::new(';', Spacing::Joint);
    semicolon.set_span(span);

    let mut err = TokenStream::new();
    err.extend([
        TokenTree::Ident(ident),
        TokenTree::Punct(bang),
        TokenTree::Group(group),
        TokenTree::Punct(semicolon),
    ]);

    err
}
