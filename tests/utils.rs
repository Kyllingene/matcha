#![allow(missing_docs)]

use matcha::*;
use proc_macro2::{TokenTree, TokenStream};

#[test]
fn main() {
    let s = r#"
        (1, 2, 3) { 1 2 3 } [123]
        a; b; c;
        [ foo ]
    "#;
    let mut stream = Stream::from(s.parse::<TokenStream>().unwrap());

    let _ = Parens::from_stream(&mut stream).unwrap();
    let _ = Braces::from_stream(&mut stream).unwrap();
    let _ = Brackets::from_stream(&mut stream).unwrap();

    let _ = Cut::<Delimited<Ident<'static>, punct::Semicolon>>::from_stream(&mut stream).unwrap();
    assert!(Cut::<Literal<'static>>::from_stream(&mut stream).map_err(|e| e.fatal).unwrap_err());

    assert_eq!(Neg(Literal("123")).match_stream(stream.view()), Ok(0));
    let mut inner = Stream::from(Group::from_stream(&mut stream).unwrap().inner);
    let _ = Greedy::<TokenTree>::from_stream(&mut inner).unwrap();

    assert_eq!(Maybe(Punct(':')).match_stream(stream.view()), Ok(0));
}
