use crate::{Error, FromStream, MatchStream, Stream, StreamLike, StreamView};
use crate::procmacro::{Delimiter, TokenStream, TokenTree};
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKind {
    // A group delimited by `()`.
    Paren,
    // A group delimited by `{}`.
    Brace,
    // A group delimited by `[]`.
    Bracket,
}

impl From<Delimiter> for GroupKind {
    fn from(d: Delimiter) -> Self {
        match d {
            Delimiter::Parenthesis | Delimiter::None => Self::Paren,
            Delimiter::Brace => Self::Brace,
            Delimiter::Bracket => Self::Bracket,
        }
    }
}

impl From<GroupKind> for Delimiter {
    fn from(g: GroupKind) -> Self {
        match g {
            GroupKind::Paren => Self::Parenthesis,
            GroupKind::Brace => Self::Brace,
            GroupKind::Bracket => Self::Bracket,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    pub kind: GroupKind,
    pub inner: TokenStream,
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Group {
    fn to_tokens(&self, stream: &mut TokenStream) {
        crate::procmacro::Group::from(self.clone()).to_tokens(stream)
    }
}

impl From<crate::procmacro::Group> for Group {
    fn from(g: crate::procmacro::Group) -> Self {
        Self {
            kind: g.delimiter().into(),
            inner: g.stream(),
        }
    }
}

impl From<&crate::procmacro::Group> for Group {
    fn from(g: &crate::procmacro::Group) -> Self {
        Self {
            kind: g.delimiter().into(),
            inner: g.stream(),
        }
    }
}

impl From<Group> for crate::procmacro::Group {
    fn from(g: Group) -> Self {
        Self::new(
            g.kind.into(),
            g.inner,
        )
    }
}

impl FromStream for Group {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = stream.peek().ok_or(Error::EndOfStream)?;
        let span = tok.span();
        let result = if let TokenTree::Group(g) = tok {
            Ok(g.into())
        } else {
            return Err(Error::Expected {
                expected: Some("(, [, or {".into()),
                got: Some(tok.to_string()),
                at: span,
            });
        };

        stream.pop();
        result
    }
}

impl MatchStream for Group {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or(None)?;
        if let TokenTree::Group(g) = tok {
            (
                GroupKind::from(g.delimiter()) == self.kind
                    && g.stream().to_string() == self.inner.to_string()
            ).then_some(1).ok_or_else(|| Some(g.to_string()))
        } else {
            Err(Some(tok.to_string()))
        }
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            GroupKind::Paren => write!(f, "( {} )", self.inner),
            GroupKind::Brace => write!(f, "{{ {} }}", self.inner),
            GroupKind::Bracket => write!(f, "[ {} ]", self.inner),
        }
    }
}

pub struct Parens;
pub struct Braces;
pub struct Brackets;

impl FromStream for Parens {
    type Output = Group;
    fn from_stream(stream: &mut Stream) -> Result<Group, Error> {
        let tok = stream.peek().ok_or(Error::EndOfStream)?;
        let span = tok.span();
        let result = if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Parenthesis
        {
            Ok(Group {
                kind: GroupKind::Paren,
                inner: g.stream(),
            })
        } else {
            return Err(Error::Expected {
                expected: Some("(".into()),
                got: Some(tok.to_string()),
                at: span,
            });
        };

        stream.pop();
        result
    }
}

impl FromStream for Braces {
    type Output = Group;
    fn from_stream(stream: &mut Stream) -> Result<Group, Error> {
        let tok = stream.peek().ok_or(Error::EndOfStream)?;
        let span = tok.span();
        let result = if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Brace
        {
            Ok(Group {
                kind: GroupKind::Brace,
                inner: g.stream(),
            })
        } else {
            return Err(Error::Expected {
                expected: Some("{".into()),
                got: Some(tok.to_string()),
                at: span,
            });
        };

        stream.pop();
        result
    }
}

impl FromStream for Brackets {
    type Output = Group;
    fn from_stream(stream: &mut Stream) -> Result<Group, Error> {
        let tok = stream.peek().ok_or(Error::EndOfStream)?;
        let span = tok.span();
        let result = if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Bracket
        {
            Ok(Group {
                kind: GroupKind::Bracket,
                inner: g.stream(),
            })
        } else {
            return Err(Error::Expected {
                expected: Some("[".into()),
                got: Some(tok.to_string()),
                at: span,
            });
        };

        stream.pop();
        result
    }
}

pub struct Delimited<T, D> {
    _items: core::marker::PhantomData<T>,
    _delimiter: core::marker::PhantomData<D>,
}

impl<T: FromStream, D: FromStream> FromStream for Delimited<T, D> {
    type Output = Vec<T::Output>;
    fn from_stream(stream: &mut Stream) -> Result<Self::Output, Error> {
        let mut items = Vec::new();

        while let Ok(item) = T::from_stream(stream) {
            items.push(item);

            if D::from_stream(stream).is_err() {
                break;
            }
        }

        Ok(items)
    }
}

pub struct Greedy<T>(core::marker::PhantomData<T>);

impl<T: FromStream> FromStream for Greedy<T> {
    type Output = T::Output;
    fn from_stream(stream: &mut Stream) -> Result<Self::Output, Error> {
        let item = T::from_stream(stream)?;
        if let Some(tt) = stream.peek() {
            let span = tt.span();
            Err(Error::Expected {
                expected: None,
                got: Some(stream.stringify()),
                at: span,
            })
        } else {
            Ok(item)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ident<'a>(pub &'a str);

impl FromStream for Ident<'static> {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = stream.peek().ok_or(Error::EndOfStream)?;
        if let TokenTree::Ident(i) = tok {
            let s = i.to_string().leak();
            stream.pop();
            Ok(Self(s))
        } else {
            Err(Error::Expected {
                expected: Some("<identifier>".into()),
                got: Some(tok.to_string()),
                at: tok.span(),
            })
        }
    }
}

impl MatchStream for Ident<'_> {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or(None)?;
        if let TokenTree::Ident(i) = tok {
            (&*i.to_string() == self.0).then_some(1).ok_or_else(|| Some(tok.to_string()))
        } else {
            Err(Some(tok.to_string()))
        }
    }
}

impl fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Ident<'_> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        use quote::TokenStreamExt;
        stream.append(proc_macro2::Ident::new(self.0, proc_macro2::Span::call_site()));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Punct(pub char);

impl FromStream for Punct {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = stream.peek().ok_or(Error::EndOfStream)?;
        if let TokenTree::Punct(p) = tok {
            let c = p.as_char();
            stream.pop();
            Ok(Self(c))
        } else {
            Err(Error::Expected {
                expected: Some("<punctuation>".into()),
                got: Some(tok.to_string()),
                at: tok.span(),
            })
        }
    }
}

impl MatchStream for Punct {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or(None)?;
        if let TokenTree::Punct(p) = tok {
            (p.as_char() == self.0).then_some(1).ok_or_else(|| Some(tok.to_string()))
        } else {
            Err(Some(tok.to_string()))
        }
    }
}

impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Punct {
    fn to_tokens(&self, stream: &mut TokenStream) {
        use quote::TokenStreamExt;
        stream.append(proc_macro2::Punct::new(self.0, proc_macro2::Spacing::Alone));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Literal<'a>(pub &'a str);

impl FromStream for Literal<'static> {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = stream.peek().ok_or(Error::EndOfStream)?;
        if let TokenTree::Literal(l) = tok {
            let s = l.to_string().leak();
            stream.pop();
            Ok(Self(s))
        } else {
            Err(Error::Expected {
                expected: Some("<literal>".into()),
                got: Some(tok.to_string()),
                at: tok.span(),
            })
        }
    }
}

impl MatchStream for Literal<'_> {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or(None)?;
        if let TokenTree::Literal(l) = tok {
            (&*l.to_string() == self.0).then_some(1).ok_or_else(|| Some(tok.to_string()))
        } else {
            Err(Some(tok.to_string()))
        }
    }
}

impl fmt::Display for Literal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Literal<'_> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        use quote::TokenStreamExt;
        use core::str::FromStr;
        stream.append(proc_macro2::Literal::from_str(self.0).expect("invalid literal"));
    }
}

pub mod punct {
    use crate::{FromStream, MatchStream, StreamLike, StreamView, Stream, Error};
    use crate::procmacro::TokenTree;
    use core::fmt;

    macro_rules! impl_punct {
        ($(
            $name:ident: $p:literal
        ),+ $(,)?) => {$(
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $name;

            impl FromStream for $name {
                type Output = Self;
                fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
                    let tok = stream.peek().ok_or(Error::EndOfStream)?;
                    if let TokenTree::Punct(p) = tok
                        && p.as_char() == $p {
                        stream.pop();
                        Ok(Self)
                    } else {
                        Err(Error::Expected {
                            expected: Some($p.to_string()),
                            got: Some(tok.to_string()),
                            at: tok.span(),
                        })
                    }
                }
            }

            impl MatchStream for $name {
                fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
                where
                    S: StreamLike,
                {
                    let tok = stream.peek().ok_or(None)?;
                    if let TokenTree::Punct(p) = tok {
                        (p.as_char() == $p).then_some(1).ok_or_else(|| Some(tok.to_string()))
                    } else {
                        Err(Some(tok.to_string()))
                    }
                }
            }

            #[cfg(feature = "quote")]
            impl quote::ToTokens for $name {
                fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
                    use quote::TokenStreamExt;
                    stream.append(proc_macro2::Punct::new($p, proc_macro2::Spacing::Alone));
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $p.fmt(f)
                }
            }
        )+};
    }

    impl_punct! {
        Slash: '/',
        Bar: '|',
        Colon: ':',
        Semicolon: ';',
        Quote: '"',
        Apos: '\'',
        Comma: ',',
        Less: '<',
        Period: '.',
        Greater: '>',
        Backslash: '\\',
        Question: '?',

        Tilde: '~',
        Backtick: '`',
        Bang: '!',
        At: '@',
        Hash: '#',
        Dollar: '$',
        Percent: '%',
        Up: '^',
        Amp: '&',
        Star: '*',

        Dash: '-',
        Underscore: '_',
        Plus: '+',
        Equals: '=',
    }
}
