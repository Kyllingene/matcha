use crate::{Error, FromStream, StreamLike};

/// A helper for parsing zero or more items.
///
/// See also [`RepeatPlus`].
pub type RepeatAny<T> = Vec<T>;

/// A helper for parsing zero or more items.
///
/// See also [`RepeatAny`].
pub struct RepeatPlus<T>(core::marker::PhantomData<T>);

impl<T: FromStream> FromStream for RepeatPlus<T> {
    type Output = Vec<T::Output>;
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike,
    {
        let mut items = vec![T::from_stream(stream)?];
        // Option handles fatal errors for us
        while let Some(item) = Option::<T>::from_stream(stream)? {
            items.push(item);
        }
        Ok(items)
    }
}
