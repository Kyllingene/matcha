use crate::{Error, FromStream, StreamLike};

/// A helper for parsing zero or more items.
pub type RepeatAny<T> = Vec<T>;

/// A helper for parsing zero or more items.
pub struct RepeatPlus<T>(core::marker::PhantomData<T>);

impl<T: FromStream> FromStream for RepeatPlus<T> {
    type Output = Vec<T::Output>;
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike + ?Sized,
    {
        let mut items = vec![T::from_stream(stream)?];
        // Option handles fatal errors for us
        while let Some(item) = Option::<T>::from_stream(stream)? {
            items.push(item);
        }
        Ok(items)
    }
}

/// A helper for parsing N or more items.
pub struct RepeatAtLeast<T, const N: usize>(core::marker::PhantomData<[T; N]>);

impl<T: FromStream, const N: usize> FromStream for RepeatAtLeast<T, N> {
    type Output = Vec<T::Output>;
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike + ?Sized,
    {
        let mut items = Vec::with_capacity(N);
        for _ in 0..N {
            items.push(T::from_stream(stream)?);
        }
        // Option handles fatal errors for us
        while let Some(item) = Option::<T>::from_stream(stream)? {
            items.push(item);
        }
        Ok(items)
    }
}

/// A helper for parsing `0..=N` items.
pub struct RepeatAtMost<T, const N: usize>(core::marker::PhantomData<[T; N]>);

impl<T: FromStream, const N: usize> FromStream for RepeatAtMost<T, N> {
    type Output = Vec<T::Output>;
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike + ?Sized,
    {
        let mut items = Vec::new();
        for _ in 0..=N {
            // Option handles fatal errors for us
            let Some(item) = Option::<T>::from_stream(stream)? else {
                break
            };

            items.push(item);
        }

        Ok(items)
    }
}

/// A helper for parsing `N..=M` items.
pub struct RepeatRange<T, const N: usize, const M: usize>(core::marker::PhantomData<([T; N], [T; M])>);

impl<T: FromStream, const N: usize, const M: usize> FromStream for RepeatRange<T, N, M> {
    type Output = Vec<T::Output>;
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike + ?Sized,
    {
        let mut items = Vec::with_capacity(N);
        for _ in 0..N {
            items.push(T::from_stream(stream)?);
        }

        for _ in N..M {
            // Option handles fatal errors for us
            let Some(item) = Option::<T>::from_stream(stream)? else {
                break
            };

            items.push(item);
        }

        debug_assert!(items.len() >= N && items.len() <= M);
        Ok(items)
    }
}
