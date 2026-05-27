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
