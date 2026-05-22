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

mod impls;
mod stream;
mod utils;

pub use stream::{Error, DiagDisplay, ErrorText, FromStream, MatchStream, Stream, StreamLike, StreamView};
pub use utils::{
    Braces, Brackets, Delimited, Greedy, Group, GroupKind, Ident, Literal, Maybe, Parens, Punct,
    punct,
};

/// Break down a [token stream](procmacro::TokenStream) into its constituents by
/// applying a pattern.
///
/// Matches a pattern to a token stream, parsing and binding variables along
/// the way.
///
/// # Patterns
///
/// The first argument is a [`Stream`] (or `&mut Stream`). Every subsequent
/// argument is an arm in the pattern
///
/// There are three kinds of "arms" in a pattern:
///  - Match arms: `= expr`, where `expr` is any expression evaluating to a type
///    implementing `MatchTokens`.
///  - Bind arms: `name => type`, where `type` is any type implementing
///    [`FromStream`]. The resulting value will be bound to `name` in the
///    calling namespace.
///  - Nesting arms: `( ... )`, `[ ... ]`, and `{ ... }`. These define a
///    recursion into a group delimited by the given arms. These can contain any
///    number of other match arms, including further nesting.
///
/// If a pattern fails to match or bind at any step, it will return an error
/// from the enclosing function.
///
/// # Errors
///
/// Must be called in a function returning <code>Result<_, [Error]></code>. Will
/// return errors it encounters.
///
/// # Examples
///
/// ```
/// # use matcha::procmacro::TokenStream;
/// # use matcha::{FromStream, Error, Stream, Ident, Brackets, Literal, GroupKind, punct, decompose};
/// # fn main() -> Result<(), Error> {
/// let input = "extern \"C\" fn foo(x: [u8; 5], y: u32, )".parse::<TokenStream>().unwrap();
/// let mut stream = Stream::from(input);
/// decompose! {
///     // the input stream; must be borrowable as `&mut Stream`
///     stream;
///
///     // matches an expression; can be anything that implements `MatchStream`
///     = Ident("extern");
///     
///     // tries to parse a type; can be anything that implements `FromStream`
///     abi: Literal;
///
///     = Ident("fn");
///     name: Ident;
///
///     // parses a group delimited by parenthesis
///     (
///         arg1_name: Ident;
///
///         // several convenience types are provided;
///         // this is equivalent to `Punct(':')`
///         = punct::Colon;
///
///         // binds a group delimited by `[]` without parsing the internals
///         arg1_type: Brackets;
///         
///         = punct::Comma;
///
///         arg2_name: Ident;
///         = punct::Colon;
///         arg2_type: Ident;
///
///         // will consume a trailing comma; if a group has any tokens left
///         // unparsed, it raises an error
///         _: Option<punct::Comma>;
///     );
///
///     // just like groups, if there were any tokens left unparsed in the
///     // stream, it would return an error
/// }
///
/// assert_eq!(abi.0, "\"C\"");
/// assert_eq!(name.0, "foo");
///
/// assert_eq!(arg1_name.0, "x");
/// assert_eq!(arg1_type.kind, GroupKind::Bracket);
/// assert_eq!(arg1_type.inner.to_string(), "u8 ; 5");
///
/// assert_eq!(arg2_name.0, "y");
/// assert_eq!(arg2_type.0, "u32");
/// # Ok(()) }
/// ```
#[macro_export]
macro_rules! decompose {
    ($($t:tt)*) => {
        $crate::__decompose_inner! {$($t)*}
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __decompose_inner {
    ( @nongreedy
        $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        ;)+
    ) => { $(
        $(
            let group = <$crate::Parens as $crate::FromStream>::from_stream(&mut $input_stream)?;
            let mut stream = $crate::Stream::from(group.inner);
            $crate::__decompose_inner! {
                stream;

                $($paren_group)*
            }
        )?

        $(
            let group = <$crate::Braces as $crate::FromStream>::from_stream(&mut $input_stream)?;
            let mut stream = $crate::Stream::from(group.inner);
            $crate::__decompose_inner! {
                stream;

                $($brace_group)*
            }
        )?

        $(
            let group = <$crate::Brackets as $crate::FromStream>::from_stream(&mut $input_stream)?;
            let mut stream = $crate::Stream::from(group.inner);
            $crate::__decompose_inner! {
                stream;

                $($bracket_group)*
            }
        )?

        $(
            $( let $bind_name = )?
                <$bind_type as $crate::FromStream>::from_stream(&mut $input_stream)?;
        )?
        $(
            let span = $input_stream.peek().map(|tt| tt.span()).unwrap_or($crate::procmacro::Span::call_site());
            match <_ as $crate::MatchStream>::match_stream(&$match_expr, $input_stream.view()) {
                Ok(n) => $input_stream.skip(n),
                Err(s) => {
                    return Err($crate::Error {
                        expected: $crate::ErrorText::Backticks($crate::DiagDisplay::diag_string(&$match_expr)),
                        got: match s {
                            Some(s) => $crate::ErrorText::Backticks(s),
                            None => $crate::ErrorText::EndOfStream,
                        },
                        at: span,
                    });
                }
            }
        )?
    )+ };

    ( @greedy
        $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        ;)+
    ) => {
        $crate::__decompose_inner! { @nongreedy $input_stream; $(
            $(( $($paren_group)* ))?
            $({ $($brace_group)* })?
            $([ $($bracket_group)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name)? : $bind_type )?
            $( = $match_expr )?
        ;)+ }

        if let Some(tt) = $input_stream.peek() {
            let span = tt.span();
            return Err($crate::Error {
                expected: $crate::ErrorText::EndOfStream,
                got: $crate::ErrorText::Backticks($input_stream.stringify()),
                at: span,
            });
        }
    };

    (
        in $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        ;)+
    ) => {
        $crate::__decompose_inner! { @nongreedy $input_stream; $(
            $(( $($paren_group)* ))?
            $({ $($brace_group)* })?
            $([ $($bracket_group)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name)? : $bind_type )?
            $( = $match_expr )?
        ;)+ }
    };

    (
        in $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        );+
    ) => {
        $crate::__decompose_inner! { @nongreedy $input_stream; $(
            $(( $($paren_group)* ))?
            $({ $($brace_group)* })?
            $([ $($bracket_group)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name)? : $bind_type )?
            $( = $match_expr )?
        ;)+ }
    };

    (
        $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        ;)+
    ) => {
        $crate::__decompose_inner! { @greedy $input_stream; $(
            $(( $($paren_group)* ))?
            $({ $($brace_group)* })?
            $([ $($bracket_group)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name)? : $bind_type )?
            $( = $match_expr )?
        ;)+ }
    };

    (
        $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        );+
    ) => {
        $crate::__decompose_inner! { @greedy $input_stream; $(
            $(( $($paren_group)* ))?
            $({ $($brace_group)* })?
            $([ $($bracket_group)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name)? : $bind_type )?
            $( = $match_expr )?
        ;)+ }
    };
}
