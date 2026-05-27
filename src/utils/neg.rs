use crate::{DiagDisplay, MatchStream, StreamLike, StreamView};

/// A helper for matching the inverse of a pattern.
///
/// When the inner pattern matches, returns `Err(None)`; else, returns `Ok(0)`.
pub struct Neg<T>(pub T);

impl<T: MatchStream> MatchStream for Neg<T> {
    fn match_stream<S>(&self, stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        match self.0.match_stream(stream) {
            Ok(_) => Err(None),
            Err(_) => Ok(0),
        }
    }
}

impl<T: DiagDisplay> DiagDisplay for Neg<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "!")?;
        self.0.fmt(f)
    }
}
