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

            $( = $bind_name:ident $bind_type:ty )?
            $( > $eq_expr:expr )?
        ),+
    ) => {$(
        $(
            let group = match $crate::Parens::from_stream(&mut $input_stream) {
                Ok(x) => x,
                Err(e) => panic!("{e}"),
            };
            let mut stream = $crate::Stream::from(group.inner);
            $crate::decompose! {
                stream;

                $($paren_group)*
            }
        )?

        $(
            let group = match $crate::Braces::from_stream(&mut $input_stream) {
                Ok(x) => x,
                Err(e) => panic!("{e}"),
            };
            let mut stream = $crate::Stream::from(group.inner);
            $crate::decompose! {
                stream;

                $($brace_group)*
            }
        )?

        $(
            let group = match $crate::Brackets::from_stream(&mut $input_stream) {
                Ok(x) => x,
                Err(e) => panic!("{e}"),
            };
            let mut stream = $crate::Stream::from(group.inner);
            $crate::decompose! {
                stream;

                $($bracket_group)*
            }
        )?

        $(
            let $bind_name = match <$bind_type as $crate::FromStream>::from_stream(&mut $input_stream) {
                Ok(x) => x,
                Err(e) => panic!("{e}"),
            };
        )?

        $(
            match <_ as $crate::MatchStream>::match_stream(&$eq_expr, $input_stream.view()) {
                Ok(n) => $input_stream.skip(n),
                Err(s) => panic!("expected `{}`, found `{s}`", $eq_expr),
            }
        )?

    )+

        if !$input_stream.is_empty() {
            panic!("unexpected input: `{}`", $input_stream.stringify());
        }
    };

    (
        $input_stream:expr;
        $(
            $(( $($paren_group:tt)* ))?
            $({ $($brace_group:tt)* })?
            $([ $($bracket_group:tt)* ])?

            $( = $bind_name:ident $bind_type:ty )?
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
