use crate::procmacro::{Span, TokenTree};
use crate::{DiagDisplay, Error, ErrorText, FromStream, MatchStream, StreamLike, StreamView, MatchResult};

/// An identifier.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Ident {
    pub ident: String,
    pub span: Span,
}

impl FromStream for Ident {
    type Output = Self;
    fn from_stream<S>(stream: &mut S) -> Result<Self, Error>
    where
        S: StreamLike + ?Sized,
    {
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Plain("ident".into())));
            }
        };
        if let TokenTree::Ident(i) = tok {
            let ident = i.to_string();
            let span = i.span();
            stream.pop();
            Ok(Self { ident, span })
        } else {
            Err(Error::new(
                ErrorText::Plain("ident".into()),
                ErrorText::Backticks(tok.to_string()),
                tok.span(),
            ))
        }
    }
}

impl MatchStream for Ident {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> MatchResult
    where
        S: StreamLike + ?Sized,
    {
        let tok = stream.peek_or()?;
        if let TokenTree::Ident(i) = tok {
            #[cfg(feature = "proc-macro2")]
            {
                (*i == self.ident)
                    .then_some(1)
                    .ok_or_else(|| (Some(tok.to_string()), tok.span()))
            }

            #[cfg(not(feature = "proc-macro2"))]
            {
                (i.to_string() == self.ident)
                    .then_some(1)
                    .ok_or_else(|| (Some(tok.to_string()), tok.span()))
            }
        } else {
            Err((Some(tok.to_string()), tok.span()))
        }
    }
}

impl core::fmt::Display for Ident {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.ident)
    }
}

impl DiagDisplay for Ident {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Ident {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use quote::TokenStreamExt;
        stream.append(proc_macro2::Ident::new(&self.ident, self.span));
    }
}

impl core::cmp::PartialEq for Ident {
    fn eq(&self, rhs: &Self) -> bool {
        self.ident == rhs.ident
    }
}

impl core::cmp::PartialEq<str> for Ident {
    fn eq(&self, rhs: &str) -> bool {
        &*self.ident == rhs
    }
}

impl core::cmp::PartialEq<&str> for Ident {
    fn eq(&self, rhs: &&str) -> bool {
        &*self.ident == *rhs
    }
}

impl core::cmp::Eq for Ident {}

impl core::hash::Hash for Ident {
    fn hash<H>(&self, h: &mut H)
    where
        H: core::hash::Hasher,
    {
        self.ident.hash(h);
    }
}
