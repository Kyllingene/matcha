use crate::{FromStream, StreamLike, Error};

/// A helper for committing to a parse tree.
///
/// Turns all subsequent parse errors into fatal ones.
pub struct Cut<T>(pub T);

impl<T: FromStream> FromStream for Cut<T> {
    type Output = T::Output;
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike,
    {
        T::from_stream(stream).map_err(Error::fatal)
    }
}
