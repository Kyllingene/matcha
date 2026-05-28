#![allow(missing_docs)]

use matcha::*;
use proc_macro2::{TokenStream, TokenTree};

#[test]
fn main() {
    let s = r#"
        (1, 2, 3) { 1 2 3 } []
        a; b; c;
        [ foo ]
    "#;
    let mut stream = Stream::from(s.parse::<TokenStream>().unwrap());

    let g1 = Parens::from_stream(&mut stream).unwrap();
    assert_eq!(g1.kind, GroupKind::Paren);
    let g2 = Braces::from_stream(&mut stream).unwrap();
    assert_eq!(g2.kind, GroupKind::Brace);
    let g3 = Brackets::from_stream(&mut stream).unwrap();
    assert_eq!(g3.kind, GroupKind::Bracket);

    let mut g2_stream = Stream::from(g2.inner);
    let lits = RepeatAny::<Literal>::from_stream(&mut g2_stream.view()).unwrap();
    assert_eq!(lits, ["1", "2", "3"]);
    let lits = RepeatPlus::<Literal>::from_stream(&mut g2_stream.view()).unwrap();
    assert_eq!(lits, ["1", "2", "3"]);
    let lits = RepeatRange::<Literal, 2, 4>::from_stream(&mut g2_stream.view()).unwrap();
    assert_eq!(lits, ["1", "2", "3"]);
    let lits = RepeatRange::<Literal, 0, 2>::from_stream(&mut g2_stream.view()).unwrap();
    assert_eq!(lits, ["1", "2"]);

    let mut g3_stream = Stream::from(g3.inner);
    let lits = RepeatAny::<Literal>::from_stream(&mut g3_stream.view()).unwrap();
    assert_eq!(lits, [""; 0]);
    assert!(RepeatPlus::<Literal>::from_stream(&mut g3_stream.view()).is_err());
    let lits = RepeatRange::<Literal, 0, 4>::from_stream(&mut g3_stream.view()).unwrap();
    assert_eq!(lits, [""; 0]);
    assert!(RepeatRange::<Literal, 2, 4>::from_stream(&mut g3_stream.view()).is_err());

    let idents = Cut::<Delimited<Ident, punct::Semicolon>>::from_stream(&mut stream).unwrap();
    assert_eq!(idents, ["a", "b", "c"]);

    assert!(
        Cut::<Literal>::from_stream(&mut stream)
            .map_err(|e| e.fatal)
            .unwrap_err()
    );

    assert_eq!(Neg("123").match_stream(stream.view()), Ok(0));
    let mut inner = Stream::from(Group::from_stream(&mut stream).unwrap().inner);

    let tt = Greedy::<TokenTree>::from_stream(&mut inner).unwrap();
    assert_eq!(tt.to_string(), "foo");

    assert_eq!(
        Maybe(Punct {
            ch: ':',
            span: proc_macro2::Span::call_site()
        })
        .match_stream(stream.view()),
        Ok(0)
    );
}
