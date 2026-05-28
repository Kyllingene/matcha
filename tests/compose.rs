#![allow(missing_docs)]

use matcha::*;
use proc_macro2::TokenStream;

compose! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Foo<T, !const N: usize>
    where [
        T: FromStream<Output = T>,
        T::Output: std::fmt::Debug + Clone + std::cmp::PartialEq,
    ] {
        x: Ident,
        = punct::Colon,
        ;^,
        y: [T; N],
        z: Option<Ident> as Option<Group> => { foo_z(z)? },
    }
}

fn foo_z(input: Option<Group>) -> Result<Option<Ident>, Error> {
    let Some(group) = input else {
        return Ok(None);
    };

    let mut stream = Stream::from(group.inner);
    decompose! {
        stream;
        name: Ident;
    }

    Ok(Some(name))
}

compose! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Bar {
        A(Foo<Literal, 3>),
        B(Ident),
        C(punct::Semicolon),
    }
}

#[test]
fn compose_struct() {
    let s = "foo: 1 2 3 (bar)";
    let input = s.parse::<TokenStream>().expect("invalid test input");

    let foo = Foo::<Literal, 3>::from_stream(&mut Stream::from(input)).unwrap();
    assert_eq!(foo.x.ident, "foo");
    assert_eq!(foo.y, ["1", "2", "3"]);
    assert_eq!(foo.z.unwrap(), "bar");
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

    let lit = |s: &str| Literal {
        literal: s.into(),
        span: proc_macro2::Span::call_site(),
    };
    assert_eq!(
        b1,
        Bar::A(Foo {
            x: Ident {
                ident: "foo".into(),
                span: proc_macro2::Span::call_site()
            },
            y: [lit("1"), lit("2"), lit("3")],
            z: None,
        })
    );
    assert_eq!(
        b2,
        Bar::B(Ident {
            ident: "bar".into(),
            span: proc_macro2::Span::call_site()
        })
    );
    assert_eq!(b3, Bar::C(punct::Semicolon));

    assert!(!b4.unwrap_err().fatal);
    assert!(b5.unwrap_err().fatal);
}
