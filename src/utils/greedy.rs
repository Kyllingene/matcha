use crate::{Error, ErrorText, FromStream, StreamLike};

/// A helper for ensuring a given item is the last in the stream.
///
/// Errors if the stream is non-empty after parsing the item (`T`).
pub struct Greedy<T>(core::marker::PhantomData<T>);

impl<T: FromStream> FromStream for Greedy<T> {
    type Output = T::Output;

    #[cfg_attr(
        all(not(feature = "proc-macro2"), feature = "proc-macro2-span-locations"),
        expect(unused_variables, reason = "`span` isn't used without span locations")
    )]
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike,
    {
        let item = T::from_stream(stream)?;
        if let Some(tt) = stream.peek() {
            let span = tt.span();
            Err(Error::new(
                ErrorText::EndOfStream,
                ErrorText::Backticks(stream.stringify()),
                span,
            ))
        } else {
            Ok(item)
        }
    }
}
