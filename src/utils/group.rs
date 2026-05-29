use crate::procmacro::{Delimiter, Span, TokenStream, TokenTree};
use crate::{DiagDisplay, Error, ErrorText, FromStream, MatchStream, StreamLike, StreamView, MatchResult};

/// The delimiters used by a [`Group`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKind {
    /// A group delimited by `()`.
    Paren,
    /// A group delimited by `{}`.
    Brace,
    /// A group delimited by `[]`.
    Bracket,
}

impl GroupKind {
    /// The opening delimiter for this kind of group.
    pub const fn open(self) -> char {
        match self {
            Self::Paren => '(',
            Self::Brace => '{',
            Self::Bracket => '[',
        }
    }

    /// The closing delimiter for this kind of group.
    pub const fn close(self) -> char {
        match self {
            Self::Paren => ')',
            Self::Brace => '}',
            Self::Bracket => ']',
        }
    }
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

/// A group of tokens.
#[derive(Debug, Clone)]
pub struct Group {
    /// The delimiters.
    pub kind: GroupKind,
    /// The inner tokens, not including the delimiters.
    pub inner: TokenStream,
    /// The whole span of the original group.
    ///
    /// ```txt
    /// ( ... )
    /// ^^^^^^^
    /// ```
    pub span: Span,
    /// The span of the opening delimiter.
    ///
    /// ```txt
    /// ( ... )
    /// ^
    /// ```
    pub span_open: Span,
    /// The span of the closing delimiter.
    ///
    /// ```txt
    /// ( ... )
    ///       ^
    /// ```
    pub span_close: Span,
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Group {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        #[cfg(feature = "proc-macro2")]
        {
            crate::procmacro::Group::from(self.clone()).to_tokens(stream)
        }
        #[cfg(not(feature = "proc-macro2"))]
        {
            let delimiter = match self.kind {
                GroupKind::Paren => proc_macro2::Delimiter::Parenthesis,
                GroupKind::Brace => proc_macro2::Delimiter::Brace,
                GroupKind::Bracket => proc_macro2::Delimiter::Bracket,
            };
            proc_macro2::Group::new(delimiter, self.inner.clone().into()).to_tokens(stream)
        }
    }
}

impl From<crate::procmacro::Group> for Group {
    fn from(g: crate::procmacro::Group) -> Self {
        Self {
            kind: g.delimiter().into(),
            inner: g.stream(),
            span: g.span(),
            span_open: g.span_open(),
            span_close: g.span_close(),
        }
    }
}

impl From<&crate::procmacro::Group> for Group {
    fn from(g: &crate::procmacro::Group) -> Self {
        Self {
            kind: g.delimiter().into(),
            inner: g.stream(),
            span: g.span(),
            span_open: g.span_open(),
            span_close: g.span_close(),
        }
    }
}

impl From<Group> for crate::procmacro::Group {
    fn from(g: Group) -> Self {
        let mut new = Self::new(g.kind.into(), g.inner);
        new.set_span(g.span);
        new
    }
}

impl FromStream for Group {
    type Output = Self;
    fn from_stream<S>(stream: &mut S) -> Result<Self, Error>
    where
        S: StreamLike,
    {
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Plain("(, [, or {".into())));
            }
        };
        let span = tok.span();
        let result = if let TokenTree::Group(g) = tok {
            Ok(g.into())
        } else {
            return Err(Error::new(
                ErrorText::Plain("`(`, `[`, or `{`".into()),
                ErrorText::Backticks(tok.to_string()),
                span,
            ));
        };

        stream.pop();
        result
    }
}

impl MatchStream for Group {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> MatchResult
    where
        S: StreamLike,
    {
        let tok = stream.peek_or()?;
        if let TokenTree::Group(g) = tok {
            (GroupKind::from(g.delimiter()) == self.kind
                && g.stream().to_string() == self.inner.to_string())
            .then_some(1)
            .ok_or_else(|| (Some(g.to_string()), g.span()))
        } else {
            Err((Some(tok.to_string()), tok.span()))
        }
    }
}

impl core::fmt::Display for Group {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            GroupKind::Paren => write!(f, "( {} )", self.inner),
            GroupKind::Brace => write!(f, "{{ {} }}", self.inner),
            GroupKind::Bracket => write!(f, "[ {} ]", self.inner),
        }
    }
}

impl DiagDisplay for Group {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

/// A helper for parsing a [`Group`] with [`GroupKind::Paren`].
pub struct Parens;
/// A helper for parsing a [`Group`] with [`GroupKind::Brace`].
pub struct Braces;
/// A helper for parsing a [`Group`] with [`GroupKind::Bracket`].
pub struct Brackets;

fn parse_group(kind: GroupKind, stream: &mut impl StreamLike) -> Result<Group, Error> {
    let tok = match stream.peek() {
        Some(tt) => tt,
        None => {
            return Err(stream.err_eos(ErrorText::Backticks("(".into())));
        }
    };
    let span = tok.span();
    let result = if let TokenTree::Group(g) = tok
        && g.delimiter() != Delimiter::None
    {
        let g_kind = GroupKind::from(g.delimiter());
        if g_kind == kind {
            Ok(Group {
                kind,
                inner: g.stream(),
                span: g.span(),
                span_open: g.span_open(),
                span_close: g.span_close(),
            })
        } else {
            Err(Error::new(
                ErrorText::Backticks(kind.open().into()),
                ErrorText::Backticks(g_kind.open().into()),
                g.span_open(),
            ))
        }
    } else {
        return Err(Error::new(
            ErrorText::Backticks(kind.open().into()),
            ErrorText::Backticks(tok.to_string()),
            span,
        ));
    };

    stream.pop();
    result
}

impl FromStream for Parens {
    type Output = Group;
    fn from_stream<S>(stream: &mut S) -> Result<Group, Error>
    where
        S: StreamLike,
    {
        parse_group(GroupKind::Paren, stream)
    }
}

impl FromStream for Braces {
    type Output = Group;
    fn from_stream<S>(stream: &mut S) -> Result<Group, Error>
    where
        S: StreamLike,
    {
        parse_group(GroupKind::Brace, stream)
    }
}

impl FromStream for Brackets {
    type Output = Group;
    fn from_stream<S>(stream: &mut S) -> Result<Group, Error>
    where
        S: StreamLike,
    {
        parse_group(GroupKind::Bracket, stream)
    }
}
