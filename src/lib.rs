extern crate proc_macro;

mod impls;
mod stream;
mod utils;

pub use stream::{Error, FromStream, MatchStream, Stream, StreamLike, StreamView};
pub use utils::{Group, GroupKind, Ident, Literal, Punct, Parens, Braces, Brackets, Delimited, Greedy, punct};

#[macro_export]
macro_rules! decompose {
    (
        $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?

            $( = $bind_name:pat = $bind_type:ty )?
            $( > $eq_expr:expr )?
        ),+
    ) => {$(
        $(
            let group = $crate::Parens::from_stream(&mut $input_stream)?;
            let mut stream = $crate::Stream::from(group.inner);
            $crate::decompose! {
                stream;

                $($paren_group)*
            }
        )?

        $(
            let group = $crate::Braces::from_stream(&mut $input_stream)?;
            let mut stream = $crate::Stream::from(group.inner);
            $crate::decompose! {
                stream;

                $($brace_group)*
            }
        )?

        $(
            let group = $crate::Brackets::from_stream(&mut $input_stream)?;
            let mut stream = $crate::Stream::from(group.inner);
            $crate::decompose! {
                stream;

                $($bracket_group)*
            }
        )?

        $(
            let $bind_name = <$bind_type as $crate::FromStream>::from_stream(&mut $input_stream)?;
        )?

        $(
            let span = $input_stream.peek().map(|tt| tt.span()).unwrap_or(::proc_macro::Span::call_site());
            match <_ as $crate::MatchStream>::match_stream(&$eq_expr, $input_stream.view()) {
                Ok(n) => $input_stream.skip(n),
                Err(s) => {
                    return Err($crate::Error::Expected {
                        expected: Some($eq_expr.to_string()),
                        got: s,
                        at: span,
                    });
                }
            }
        )?

    )+

        if let Some(tt) = $input_stream.peek() {
            let span = tt.span();
            return Err($crate::Error::Expected {
                expected: None,
                got: Some($input_stream.stringify()),
                at: span,
            });
        }
    };

    (
        $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?

            $( = $bind_name:pat = $bind_type:ty )?
            $( > $eq_expr:expr )?
        ),+ ,
    ) => {
        $crate::decompose! {
            $input_stream;
            $(
                $(( $($paren_group)* ))?
                $({ $($brace_group)* })?
                $([ $($bracket_group)* ])?

                $( = $bind_name:ident $bind_type:ty )?
                $( > $eq_expr )?
            ),+
        }
    };
}
