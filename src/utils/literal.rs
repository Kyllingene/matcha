use crate::procmacro::{TokenTree, Span};
use crate::{FromStream, StreamLike, StreamView, DiagDisplay, MatchStream, Error, ErrorText};

/// A piece of punctuation (e.g. `123`, `bool`, `"foo"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Literal<'a>(pub &'a str);

impl FromStream for Literal<'static> {
    type Output = Self;
    fn from_stream<S>(stream: &mut S) -> Result<Self, Error>
    where
        S: StreamLike,
    {
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
            Err(Error::new(
                ErrorText::Plain("literal".into()),
                ErrorText::Backticks(tok.to_string()),
                tok.span(),
            ))
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

impl core::fmt::Display for Literal<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl DiagDisplay for Literal<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
