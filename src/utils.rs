use crate::procmacro::{Delimiter, TokenStream, TokenTree};
use crate::{DiagDisplay, Error, ErrorText, FromStream, MatchStream, Stream, StreamLike, StreamView};
use core::fmt;

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
        Self::new(g.kind.into(), g.inner)
    }
}

impl FromStream for Group {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
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
            return Err(Error {
                expected: ErrorText::Plain("`(`, `[`, or `{`".into()),
                got: ErrorText::Backticks(tok.to_string()),
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
            (GroupKind::from(g.delimiter()) == self.kind
                && g.stream().to_string() == self.inner.to_string())
            .then_some(1)
            .ok_or_else(|| Some(g.to_string()))
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

impl DiagDisplay for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// A helper for parsing a [`Group`] with [`GroupKind::Paren`].
pub struct Parens;
/// A helper for parsing a [`Group`] with [`GroupKind::Brace`].
pub struct Braces;
/// A helper for parsing a [`Group`] with [`GroupKind::Bracket`].
pub struct Brackets;

impl FromStream for Parens {
    type Output = Group;
    fn from_stream(stream: &mut Stream) -> Result<Group, Error> {
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Backticks("(".into())));
            }
        };
        let span = tok.span();
        let result = if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Parenthesis
        {
            Ok(Group {
                kind: GroupKind::Paren,
                inner: g.stream(),
            })
        } else {
            return Err(Error {
                expected: ErrorText::Backticks("(".into()),
                got: ErrorText::Backticks(tok.to_string()),
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
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Backticks("{".into())));
            }
        };
        let span = tok.span();
        let result = if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Brace
        {
            Ok(Group {
                kind: GroupKind::Brace,
                inner: g.stream(),
            })
        } else {
            return Err(Error {
                expected: ErrorText::Backticks("{".into()),
                got: ErrorText::Backticks(tok.to_string()),
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
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Backticks("[".into())));
            }
        };
        let span = tok.span();
        let result = if let TokenTree::Group(g) = tok
            && g.delimiter() == Delimiter::Bracket
        {
            Ok(Group {
                kind: GroupKind::Bracket,
                inner: g.stream(),
            })
        } else {
            return Err(Error {
                expected: ErrorText::Backticks("[".into()),
                got: ErrorText::Backticks(tok.to_string()),
                at: span,
            });
        };

        stream.pop();
        result
    }
}

/// A helper for parsing a series of items.
///
/// Parses a series of items (`T`), separated by delimiters (`D`), with an
/// optional trailing delimiter.
///
/// Note that this doesn't parse any brackets around the items; for example, if
/// you want to parse a tuple, you'd need to layer [`Parens`].
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

/// A helper for ensuring a given item is the last in the stream.
///
/// Errors if the stream is non-empty after parsing the item (`T`).
pub struct Greedy<T>(core::marker::PhantomData<T>);

impl<T: FromStream> FromStream for Greedy<T> {
    type Output = T::Output;
    fn from_stream(stream: &mut Stream) -> Result<Self::Output, Error> {
        let item = T::from_stream(stream)?;
        if let Some(tt) = stream.peek() {
            let span = tt.span();
            Err(Error {
                expected: ErrorText::EndOfStream,
                got: ErrorText::Backticks(stream.stringify()),
                at: span,
            })
        } else {
            Ok(item)
        }
    }
}

/// A helper for optionally matching a given pattern.
pub struct Maybe<T>(pub T);

impl<T: MatchStream> MatchStream for Maybe<T> {
    fn match_stream<S>(&self, stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        Ok(self.0.match_stream(stream).unwrap_or(0))
    }
}

impl<T: DiagDisplay> DiagDisplay for Maybe<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// An identifier.
///
/// When parsed out of a stream, returns a `'static` ident, from a leaked
/// `String`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ident<'a>(pub &'a str);

impl FromStream for Ident<'static> {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Plain("ident".into())));
            }
        };
        if let TokenTree::Ident(i) = tok {
            let s = i.to_string().leak();
            stream.pop();
            Ok(Self(s))
        } else {
            Err(Error {
                expected: ErrorText::Plain("ident".into()),
                got: ErrorText::Backticks(tok.to_string()),
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
            (&*i.to_string() == self.0)
                .then_some(1)
                .ok_or_else(|| Some(tok.to_string()))
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

impl DiagDisplay for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Ident<'_> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use quote::TokenStreamExt;
        stream.append(proc_macro2::Ident::new(
            self.0,
            proc_macro2::Span::call_site(),
        ));
    }
}

/// A piece of punctuation (e.g. `!`, `.`, `:`).
///
/// See also [`punct`] for helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Punct(pub char);

impl FromStream for Punct {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Plain("punctuation".into())));
            }
        };
        if let TokenTree::Punct(p) = tok {
            let c = p.as_char();
            stream.pop();
            Ok(Self(c))
        } else {
            Err(Error {
                expected: ErrorText::Plain("punctuation".into()),
                got: ErrorText::Backticks(tok.to_string()),
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
            (p.as_char() == self.0)
                .then_some(1)
                .ok_or_else(|| Some(tok.to_string()))
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


impl DiagDisplay for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Punct {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use quote::TokenStreamExt;
        stream.append(proc_macro2::Punct::new(self.0, proc_macro2::Spacing::Alone));
    }
}

/// A piece of punctuation (e.g. `123`, `bool`, `"foo"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Literal<'a>(pub &'a str);

impl FromStream for Literal<'static> {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Plain("literal".into())));
            }
        };
        if let TokenTree::Literal(l) = tok {
            let s = l.to_string().leak();
            stream.pop();
            Ok(Self(s))
        } else {
            Err(Error {
                expected: ErrorText::Plain("literal".into()),
                got: ErrorText::Backticks(tok.to_string()),
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
            (&*l.to_string() == self.0)
                .then_some(1)
                .ok_or_else(|| Some(tok.to_string()))
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

impl DiagDisplay for Literal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Literal<'_> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use core::str::FromStr;
        use quote::TokenStreamExt;
        stream.append(proc_macro2::Literal::from_str(self.0).expect("invalid literal"));
    }
}

/// Helpers for parsing specific [`Punct`]s.
pub mod punct {
    use crate::procmacro::TokenTree;
    use crate::{Error, DiagDisplay, ErrorText, FromStream, MatchStream, Stream, StreamLike, StreamView};
    use core::fmt;

    macro_rules! impl_punct {
        ($(
            $name:ident: $p:literal = $doc:literal
        ),+ $(,)?) => {$(
            #[doc = concat!(
                "A helper for parsing ",
                $doc,
                " into a [`Punct`](crate::Punct).",
            )]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $name;

            impl FromStream for $name {
                type Output = Self;
                fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
                    let tok = match stream.peek() {
                        Some(tt) => tt,
                        None => {
                            return Err(stream.err_eos(ErrorText::Backticks($p.to_string())));
                        }
                    };
                    if let TokenTree::Punct(p) = tok
                        && p.as_char() == $p {
                        stream.pop();
                        Ok(Self)
                    } else {
                        Err(Error {
                            expected: ErrorText::Backticks($p.to_string()),
                            got: ErrorText::Backticks(tok.to_string()),
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

            impl DiagDisplay for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{self}")
                }
            }
        )+};
    }

    impl_punct! {
        Slash: '/' = "a slash (`/`)",
        Bar: '|' = "a pipe (`|`)",
        Colon: ':' = "a colon (`:`)",
        Semicolon: ';' = "a semicolon (`;`)",
        Quote: '"' = "a quote (`\"`)",
        Apos: '\'' = "an apostrophe (`'`)",
        Comma: ',' = "a comma (`,`)",
        Less: '<' = "a left angle bracket (`<`)",
        Period: '.' = "a period (`.`)",
        Greater: '>' = "a right angle bracket (`>`)",
        Backslash: '\\' = "a backslash (`\\`)",
        Question: '?' = "a question mark (`?`)",

        Tilde: '~' = "a tilde (`~`)",
        Backtick: '`' = "a backtick (`` ` ``)",
        Bang: '!' = "an exclamation mark (`!`)",
        At: '@' = "an at symbol (`@`)",
        Hash: '#' = "a pound sign (`#`)",
        Dollar: '$' = "a dollar sign (`$`)",
        Percent: '%' = "a percent sign (`%`)",
        Caret: '^' = "a caret (`^`)",
        Amp: '&' = "an ampersand (`&`)",
        Star: '*' = "an asterisk (`*`)",

        Dash: '-' = "a minus sign (`-`)",
        Underscore: '_' = "an underscore (`_`)",
        Plus: '+' = "a plus sign (`+`)",
        Equals: '=' = "an equals sign (`=`)",
    }
}
