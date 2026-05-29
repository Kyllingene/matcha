use crate::procmacro::{Span, TokenTree};
use crate::{DiagDisplay, MatchStream, StreamLike, StreamView, MatchResult};

/// A helper for matching the inverse of a pattern.
///
/// When the inner pattern matches, returns `Err(None)`; else, returns `Ok(0)`.
pub struct Neg<T>(pub T);

impl<T: MatchStream> MatchStream for Neg<T> {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> MatchResult
    where
        S: StreamLike,
    {
        let span = stream.peek().map(TokenTree::span).unwrap_or(Span::call_site());
        if self.0.match_stream(stream).is_err() {
            Ok(0)
        } else {
            Err((None, span))
        }
    }
}

impl<T: DiagDisplay> DiagDisplay for Neg<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "!")?;
        self.0.fmt(f)
    }
}
