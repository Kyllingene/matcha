//! A framework for breaking down token streams in proc macros.
//!
//! The macro [`decompose!`] is the main entrypoint for this crate. However, the
//! surrounding types/traits can be useful regardless.
//!
//! # Features
//!
//! The following features are provided (all are enabled by default).
//!
//! - `quote`: enables [`ToTokens`](quote::ToTokens) impls for various types.
//! - `proc-macro2`: uses the [`proc-macro2`](proc-macro2) crate instead of the
//!   built-in `proc-macro`.
//! - `proc-macro2-span-locations`: enables the `proc-macro2` feature
//!   `span-locations`, allowing [errors](Error) to print their locations.

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
mod impls;
mod stream;
mod utils;

pub use stream::{
    DiagDisplay, Error, ErrorText, FromStream, MatchStream, Stream, StreamLike, StreamView,
};
pub use utils::{
    Braces, Brackets, Cut, Delimited, Greedy, Group, GroupKind, Ident, Literal, Maybe, Neg, Parens,
    Punct, punct,
};
