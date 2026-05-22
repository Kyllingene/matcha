use crate::{Error, FromStream, MatchStream, Stream, StreamLike, StreamView};
use proc_macro::{Delimiter, TokenStream, TokenTree};
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

pub struct Group {
    pub kind: GroupKind,
    pub inner: TokenStream,
}

impl From<proc_macro::Group> for Group {
    fn from(g: proc_macro::Group) -> Self {
        Self {
            kind: g.delimiter().into(),
            inner: g.stream(),
        }
    }
}

impl From<&proc_macro::Group> for Group {
    fn from(g: &proc_macro::Group) -> Self {
        Self {
            kind: g.delimiter().into(),
            inner: g.stream(),
        }
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
            return Err(Error::Invalid(span));
        };

        stream.pop();
        result
    }
}

impl MatchStream for Group {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, String>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or_else(|| "<end of input>".to_string())?;
        if let TokenTree::Group(g) = tok {
            (
                GroupKind::from(g.delimiter()) == self.kind
                    && g.stream().to_string() == self.inner.to_string()
            ).then_some(1).ok_or_else(|| g.to_string())
        } else {
            return Err(tok.to_string());
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
            return Err(Error::Invalid(span));
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
            return Err(Error::Invalid(span));
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
            return Err(Error::Invalid(span));
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
            Err(Error::Invalid(tt.span()))
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
            Err(Error::Invalid(tok.span()))
        }
    }
}

impl MatchStream for Ident<'_> {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, String>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or_else(|| "<end of input>".to_string())?;
        if let TokenTree::Ident(i) = tok {
            (&*i.to_string() == self.0).then_some(1).ok_or_else(|| tok.to_string())
        } else {
            Err(tok.to_string())
        }
    }
}

impl fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
            Err(Error::Invalid(tok.span()))
        }
    }
}

impl MatchStream for Punct {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, String>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or_else(|| "<end of input>".to_string())?;
        if let TokenTree::Punct(p) = tok {
            (p.as_char() == self.0).then_some(1).ok_or_else(|| tok.to_string())
        } else {
            Err(tok.to_string())
        }
    }
}

impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
            Err(Error::Invalid(tok.span()))
        }
    }
}

impl MatchStream for Literal<'_> {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, String>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or_else(|| "<end of input>".to_string())?;
        if let TokenTree::Literal(l) = tok {
            (&*l.to_string() == self.0).then_some(1).ok_or_else(|| tok.to_string())
        } else {
            Err(tok.to_string())
        }
    }
}

impl fmt::Display for Literal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub mod punct {
    use crate::{FromStream, MatchStream, StreamLike, StreamView, Stream, Error};
    use proc_macro::TokenTree;
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
                        Err(Error::Invalid(tok.span()))
                    }
                }
            }

            impl MatchStream for $name {
                fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, String>
                where
                    S: StreamLike,
                {
                    let tok = stream.peek().ok_or_else(|| "<end of input>".to_string())?;
                    if let TokenTree::Punct(p) = tok {
                        (p.as_char() == $p).then_some(1).ok_or_else(|| tok.to_string())
                    } else {
                        Err(tok.to_string())
                    }
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
