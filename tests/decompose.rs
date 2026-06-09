#![allow(missing_docs)]

use matcha::*;
use proc_macro2::TokenStream;

#[derive(Debug, PartialEq, Eq)]
struct Arg {
    name: Ident,
    type_: Ident,
}

impl FromStream for Arg {
    type Output = Self;
    fn from_stream<S>(mut stream: &mut S) -> Result<Self, Error>
    where
        S: StreamLike + ?Sized,
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

        = "extern";
        abi: Option<Literal>;
        = "fn";
        name: Ident;

        ( args: Delimited<Arg, punct::Comma> );

        = (punct::Dash, punct::Greater);

        wrapper: Ident;
        = punct::Less;
        inner: Ident;
        = punct::Greater;

        = punct::Semicolon;
    }

    assert_eq!(abi.unwrap(), "\"C\"");
    assert_eq!(name.ident, "foo");

    assert_eq!(args.len(), 2);
    assert_eq!(args[0].name, "x");
    assert_eq!(args[0].type_, "String");
    assert_eq!(args[1].name, "y");
    assert_eq!(args[1].type_, "usize");

    assert_eq!(wrapper.ident, "Option");
    assert_eq!(inner.ident, "char");

    Ok(())
}
