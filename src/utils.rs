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

pub struct Parens;
pub struct Braces;
pub struct Brackets;

impl FromStream for Parens {
    type Output = Group;
    fn from_stream(stream: &mut Stream) -> Result<Group, Error> {
        let tok = stream.pop().ok_or(Error::EndOfStream)?;
        let span = tok.span();
        if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Parenthesis
        {
            Ok(Group {
                kind: GroupKind::Paren,
                inner: g.stream(),
            })
        } else {
            Err(Error::Invalid(span))
        }
    }
}

impl FromStream for Braces {
    type Output = Group;
    fn from_stream(stream: &mut Stream) -> Result<Group, Error> {
        let tok = stream.pop().ok_or(Error::EndOfStream)?;
        let span = tok.span();
        if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Brace
        {
            Ok(Group {
                kind: GroupKind::Brace,
                inner: g.stream(),
            })
        } else {
            Err(Error::Invalid(span))
        }
    }
}

impl FromStream for Brackets {
    type Output = Group;
    fn from_stream(stream: &mut Stream) -> Result<Group, Error> {
        let tok = stream.pop().ok_or(Error::EndOfStream)?;
        let span = tok.span();
        if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Bracket
        {
            Ok(Group {
                kind: GroupKind::Bracket,
                inner: g.stream(),
            })
        } else {
            Err(Error::Invalid(span))
        }
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
            Err(tok.to_string())
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

pub struct Ident<'a>(pub &'a str);

impl FromStream for Ident<'static> {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = stream.pop().ok_or(Error::EndOfStream)?;
        if let TokenTree::Ident(i) = tok {
            Ok(Self(i.to_string().leak()))
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

pub struct Punct<'a>(pub &'a str);

impl FromStream for Punct<'static> {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = stream.pop().ok_or(Error::EndOfStream)?;
        if let TokenTree::Punct(p) = tok {
            Ok(Self(p.to_string().leak()))
        } else {
            Err(Error::Invalid(tok.span()))
        }
    }
}

impl MatchStream for Punct<'_> {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, String>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or_else(|| "<end of input>".to_string())?;
        if let TokenTree::Punct(p) = tok {
            (&*p.to_string() == self.0).then_some(1).ok_or_else(|| tok.to_string())
        } else {
            Err(tok.to_string())
        }
    }
}

impl fmt::Display for Punct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Literal<'a>(pub &'a str);

impl FromStream for Literal<'static> {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = stream.pop().ok_or(Error::EndOfStream)?;
        if let TokenTree::Literal(l) = tok {
            Ok(Self(l.to_string().leak()))
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
