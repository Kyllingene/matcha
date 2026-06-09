use crate::{Error, FromStream, StreamLike};

/// A helper for parsing a series of items.
///
/// Parses a series of items (`T`), separated by delimiters (`D`), with an
/// optional trailing delimiter.
///
/// Note that this doesn't parse any brackets around the items; for example, if
/// you want to parse a tuple, you'd need to layer [`Parens`](crate::Parens).
pub struct Delimited<T, D> {
    _items: core::marker::PhantomData<T>,
    _delimiter: core::marker::PhantomData<D>,
}

impl<T: FromStream, D: FromStream> FromStream for Delimited<T, D> {
    type Output = Vec<T::Output>;
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike + ?Sized,
    {
        let mut items = Vec::new();
        // Option handles fatal errors for us
        while let Some(item) = Option::<T>::from_stream(stream)? {
            items.push(item);

            match D::from_stream(stream) {
                Ok(_) => {}
                Err(e) if e.fatal => return Err(e),
                Err(_) => break,
            }
        }
        Ok(items)
    }
}
