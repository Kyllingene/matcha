use crate::procmacro::{Span, TokenTree};
use crate::{DiagDisplay, Error, ErrorText, FromStream, MatchStream, StreamLike, StreamView, MatchResult};

/// A literal.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Literal {
    pub literal: String,
    pub span: Span,
}

impl FromStream for Literal {
    type Output = Self;
    fn from_stream<S>(stream: &mut S) -> Result<Self, Error>
    where
        S: StreamLike + ?Sized,
    {
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Plain("literal".into())));
            }
        };
        if let TokenTree::Literal(i) = tok {
            let literal = i.to_string();
            let span = i.span();
            stream.pop();
            Ok(Self { literal, span })
        } else {
            Err(Error::new(
                ErrorText::Plain("literal".into()),
                ErrorText::Backticks(tok.to_string()),
                tok.span(),
            ))
        }
    }
}

impl MatchStream for Literal {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> MatchResult
    where
        S: StreamLike + ?Sized,
    {
        let tok = stream.peek_or()?;
        if let TokenTree::Literal(i) = tok {
            (i.to_string() == self.literal)
                .then_some(1)
                .ok_or_else(|| (Some(tok.to_string()), tok.span()))
        } else {
            Err((Some(tok.to_string()), tok.span()))
        }
    }
}

impl core::fmt::Display for Literal {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.literal)
    }
}

impl DiagDisplay for Literal {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Literal {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use core::str::FromStr;
        use quote::TokenStreamExt;
        let mut tt = proc_macro2::Literal::from_str(&self.literal).expect("invalid literal");
        tt.set_span(self.span);
        stream.append(tt);
    }
}

impl core::cmp::PartialEq for Literal {
    fn eq(&self, rhs: &Self) -> bool {
        self.literal == rhs.literal
    }
}

impl core::cmp::PartialEq<str> for Literal {
    fn eq(&self, rhs: &str) -> bool {
        &*self.literal == rhs
    }
}

impl core::cmp::PartialEq<&str> for Literal {
    fn eq(&self, rhs: &&str) -> bool {
        &*self.literal == *rhs
    }
}

impl core::cmp::Eq for Literal {}

impl core::hash::Hash for Literal {
    fn hash<H>(&self, h: &mut H)
    where
        H: core::hash::Hasher,
    {
        self.literal.hash(h);
    }
}
