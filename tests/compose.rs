#![allow(missing_docs)]

use matcha::*;
use proc_macro2::TokenStream;

compose! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Foo<T, !const N: usize>
    where [
        T: FromStream,
        T::Output: std::fmt::Debug + Clone + std::cmp::PartialEq,
    ] {
        x: Ident<'static>,
        = punct::Colon,
        ;^,
        y: [T; N],
    }
}

compose! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Bar {
        A(Foo<Literal<'static>, 3>),
        B(Ident<'static>),
        C(punct::Semicolon),
    }
}

#[test]
fn compose_struct() {
    let s = "foo: 1 2 3";
    let input = s.parse::<TokenStream>().expect("invalid test input");

    let foo = Foo::<Literal<'static>, 3>::from_stream(&mut Stream::from(input)).unwrap();
    assert_eq!(foo.x.0, "foo");
    assert_eq!(foo.y, [Literal("1"), Literal("2"), Literal("3"),]);
}

#[test]
fn compose_enum() {
    let [i1, i2, i3, i4, i5] = ["foo: 1 2 3", "bar", ";", "456", "baz: x y z"]
        .map(|s| s.parse::<TokenStream>().expect("invalid test input"));

    let (b1, b2, b3, b4, b5) = (
        Bar::from_stream(&mut Stream::from(i1)).unwrap(),
        Bar::from_stream(&mut Stream::from(i2)).unwrap(),
        Bar::from_stream(&mut Stream::from(i3)).unwrap(),
        Bar::from_stream(&mut Stream::from(i4)),
        Bar::from_stream(&mut Stream::from(i5)),
    );

    assert_eq!(
        b1,
        Bar::A(Foo {
            x: Ident("foo"),
            y: [Literal("1"), Literal("2"), Literal("3")],
        })
    );
    assert_eq!(b2, Bar::B(Ident("bar")));
    assert_eq!(b3, Bar::C(punct::Semicolon));

    assert!(!b4.unwrap_err().fatal);
    assert!(b5.unwrap_err().fatal);
}
