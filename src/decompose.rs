/// Break down a [token stream](crate::procmacro::TokenStream) into its constituents by
/// applying a pattern.
///
/// Matches a pattern to a token stream, parsing and binding variables along
/// the way.
///
/// The first argument is a [`Stream`](crate::Stream) (or `&mut Stream`). Every
/// subsequent argument is an arm in the pattern.
///
/// The first argument may be preceded by `in` (e.g. `in stream;`). This
/// disables "greedy mode", where an error is raised if the stream is not
/// empty at the end of parsing.
///
/// If you need this behavior inside a block, consider binding against a
/// trailing `Vec<TokenTree>`.
///
/// # Patterns
///
/// There are three kinds of "arms" in a pattern:
///  - Match arms: `= expr`, where `expr` is any expression evaluating to a type
///    implementing `MatchTokens`.
///  - Bind arms: `name => type`, where `type` is any type implementing
///    [`FromStream`](crate::FromStream). The resulting value will be bound to
///    `name` in the calling namespace.
///  - Nesting arms: `( ... )`, `[ ... ]`, and `{ ... }`. These define a
///    recursion into a group delimited by the given arms. These can contain any
///    number of other match arms, including further nesting.
///
/// In addition, you can use the "cut" operator (written `^;`), to specify that
/// from this point forward, errors are non-recoverable. You can use this to
/// improve error messages by prohibiting upstream from trying to recover when
/// you encounter a hard error. See also [`Cut`](crate::Cut).
///
/// If a pattern fails to match or bind at any step, it will return an error
/// from the enclosing function.
///
/// # Errors
///
/// Must be called in a function returning
/// <code>Result<_, [Error](crate::Error)></code>. Will return errors it
/// encounters from the enclosing function.
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
///     // note that `&str` can only match one token
///     = "extern";
///     
///     // tries to parse a type; can be anything that implements `FromStream`
///     abi: Literal;
///
///     = "fn";
///     name: Ident;
///
///     // parses a group delimited by parenthesis
///     (
///         ^; // does nothing in this example, but here it is
///
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
/// assert_eq!(abi.literal, "\"C\"");
/// assert_eq!(name.ident, "foo");
///
/// assert_eq!(arg1_name.ident, "x");
/// assert_eq!(arg1_type.kind, GroupKind::Bracket);
/// assert_eq!(arg1_type.inner.to_string(), "u8 ; 5");
///
/// assert_eq!(arg2_name.ident, "y");
/// assert_eq!(arg2_type.ident, "u32");
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
            $( ^ $(@@ $cut:tt)? )?

            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        ;)+
    ) => {
            let mut cut = false;
    $(
        $( cut = true; $($cut)? )?

        $(
            let group = <$crate::Parens as $crate::FromStream>::from_stream(&mut $input_stream)
                .map_err(|e| e.with_fatal(cut))?;
            let mut stream = $crate::Stream::from(group);
            $crate::__decompose_inner! {
                stream;

                $($paren_group)*
            }
        )?

        $(
            let group = <$crate::Braces as $crate::FromStream>::from_stream(&mut $input_stream)
                .map_err(|e| e.with_fatal(cut))?;
            let mut stream = $crate::Stream::from(group);
            $crate::__decompose_inner! {
                stream;

                $($brace_group)*
            }
        )?

        $(
            let group = <$crate::Brackets as $crate::FromStream>::from_stream(&mut $input_stream)
                .map_err(|e| e.with_fatal(cut))?;
            let mut stream = $crate::Stream::from(group);
            $crate::__decompose_inner! {
                stream;

                $($bracket_group)*
            }
        )?

        $(
            $( let $bind_name = )?
                <$bind_type as $crate::FromStream>::from_stream(&mut $input_stream)
                .map_err(|e| e.with_fatal(cut))?;
        )?
        $(
            let view = $crate::StreamLike::view(&mut $input_stream);
            match <_ as $crate::MatchStream>::match_stream(&$match_expr, view) {
                Ok(n) => $crate::StreamLike::skip(&mut $input_stream, n),
                Err((s, span)) => {
                    return Err($crate::Error::new(
                        $crate::ErrorText::Backticks($crate::DiagDisplay::diag_string(&$match_expr)),
                        match s {
                            Some(s) => $crate::ErrorText::Backticks(s),
                            None => $crate::ErrorText::EndOfStream,
                        },
                        span,
                    ).with_fatal(cut));
                }
            }
        )?
    )+ };

    ( @greedy
        $input_stream:expr;
        $(
            $( ^ $(@@ $cut:tt)? )?

            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        ;)+
    ) => {
        $crate::__decompose_inner! { @nongreedy $input_stream; $(
            $( ^ $($cut)? )?

            $(( $($paren_group)* ))?
            $({ $($brace_group)* })?
            $([ $($bracket_group)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name)? : $bind_type )?
            $( = $match_expr )?
        ;)+ }

        if let Some(tt) = $crate::StreamLike::peek(&mut $input_stream) {
            let span = tt.span();
            return Err($crate::Error::new(
                $crate::ErrorText::EndOfStream,
                $crate::ErrorText::Backticks(
                    $crate::StreamLike::stringify(&mut $input_stream),
                ),
                span,
            ).fatal());
        }
    };

    (
        in $input_stream:expr;
        $(
            $( ^ $(@@ $cut:tt)? )?

            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        ;)+
    ) => {
        $crate::__decompose_inner! { @nongreedy $input_stream; $(
            $( ^ $($cut)? )?

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
            $( ^ $(@@ $cut:tt)? )?

            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        );+
    ) => {
        $crate::__decompose_inner! { @nongreedy $input_stream; $(
            $( ^ $($cut)? )?

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
            $( ^ $(@@ $cut:tt)? )?

            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        ;)+
    ) => {
        $crate::__decompose_inner! { @greedy $input_stream; $(
            $( ^ $($cut)? )?

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
            $( ^ $(@@ $cut:tt)? )?

            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name:ident)? $(_)? : $bind_type:ty )?
            $( = $match_expr:expr )?
        );+
    ) => {
        $crate::__decompose_inner! { @greedy $input_stream; $(
            $( ^ $($cut)? )?

            $(( $($paren_group)* ))?
            $({ $($brace_group)* })?
            $([ $($bracket_group)* ])?
            $(?{ $($opt_block:tt)* })?

            $( $($bind_name)? : $bind_type )?
            $( = $match_expr )?
        ;)+ }
    };
}
