use crate::procmacro::{TokenTree, Span};
use crate::{FromStream, StreamLike, StreamView, DiagDisplay, MatchStream, Error, ErrorText};

/// An identifier.
///
/// When parsed out of a stream, returns a `'static` ident, from a leaked
/// `String`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ident<'a>(pub &'a str);

impl FromStream for Ident<'static> {
    type Output = Self;
    fn from_stream<S>(stream: &mut S) -> Result<Self, Error>
    where
        S: StreamLike,
    {
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
            Err(Error::new(
                ErrorText::Plain("ident".into()),
                ErrorText::Backticks(tok.to_string()),
                tok.span(),
            ))
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

impl core::fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl DiagDisplay for Ident<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
