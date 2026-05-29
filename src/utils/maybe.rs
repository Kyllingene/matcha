use crate::{DiagDisplay, MatchStream, StreamLike, StreamView, MatchResult};

/// A helper for optionally matching a given pattern.
///
/// This is not for optionally [*parsing*](crate::FromStream) a given pattern;
/// for that, use `Option<T>`.
pub struct Maybe<T>(pub T);

impl<T: MatchStream> MatchStream for Maybe<T> {
    fn match_stream<S>(&self, stream: StreamView<'_, S>) -> MatchResult
    where
        S: StreamLike,
    {
        Ok(self.0.match_stream(stream).unwrap_or(0))
    }
}

impl<T: DiagDisplay> DiagDisplay for Maybe<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
