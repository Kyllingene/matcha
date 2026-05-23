#![allow(missing_docs)]

use matcha::*;
use proc_macro2::TokenStream;

#[derive(Debug, PartialEq, Eq)]
struct Arg {
    name: Ident<'static>,
    type_: Ident<'static>,
}

impl FromStream for Arg {
    type Output = Self;
    fn from_stream<S>(mut stream: &mut S) -> Result<Self, Error>
    where
        S: StreamLike,
    {
        decompose! {
            in stream;

            name: Ident;
            = punct::Colon;
            type_: Ident;
        }

        Ok(Self { name, type_ })
    }
}

#[test]
fn main() -> Result<(), Error> {
    let input = include_str!("decompose_input.txt")
        .parse::<TokenStream>()
        .expect("invalid test input");

    let mut stream = Stream::from(input);

    decompose! {
        stream;

        = Ident("extern");
        abi: Option<Literal>;
        = Ident("fn");
        name: Ident;

        ( args: Delimited<Arg, punct::Comma> );

        = (punct::Dash, punct::Greater);

        wrapper: Ident;
        = punct::Less;
        inner: Ident;
        = punct::Greater;

        = punct::Semicolon;
    }

    assert_eq!(abi, Some(Literal("\"C\"")));
    assert_eq!(name.0, "foo");

    assert_eq!(
        args,
        [
            Arg {
                name: Ident("x"),
                type_: Ident("String"),
            },
            Arg {
                name: Ident("y"),
                type_: Ident("usize"),
            },
        ]
    );

    assert_eq!(wrapper.0, "Option");
    assert_eq!(inner.0, "char");

    Ok(())
}
