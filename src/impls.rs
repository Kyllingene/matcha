use crate::procmacro::TokenTree;
use crate::{Error, ErrorText, DiagDisplay, FromStream, MatchStream, Stream, StreamLike, StreamView};

impl FromStream for TokenTree {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        stream.pop().ok_or(stream.err_eos(ErrorText::Token))
    }
}

impl MatchStream for TokenTree {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        match (self, stream.peek().ok_or(None)?) {
            (TokenTree::Group(lhs), TokenTree::Group(rhs)) => (lhs.to_string() == rhs.to_string())
                .then_some(1)
                .ok_or_else(|| Some(rhs.to_string())),
            (TokenTree::Ident(lhs), TokenTree::Ident(rhs)) => {
                #[cfg(feature = "proc-macro2")]
                {
                    (lhs == rhs)
                        .then_some(1)
                        .ok_or_else(|| Some(rhs.to_string()))
                }

                #[cfg(not(feature = "proc-macro2"))]
                {
                    (lhs.to_string() == rhs.to_string())
                        .then_some(1)
                        .ok_or_else(|| Some(rhs.to_string()))
                }
            }
            (TokenTree::Punct(lhs), TokenTree::Punct(rhs)) => (lhs.to_string() == rhs.to_string())
                .then_some(1)
                .ok_or_else(|| Some(rhs.to_string())),
            (TokenTree::Literal(lhs), TokenTree::Literal(rhs)) => (lhs.to_string()
                == rhs.to_string())
            .then_some(1)
            .ok_or_else(|| Some(rhs.to_string())),
            (_, rhs) => Err(Some(rhs.to_string())),
        }
    }
}

impl DiagDisplay for TokenTree {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

impl<T> FromStream for Option<T>
where
    T: FromStream,
{
    type Output = Option<T::Output>;
    fn from_stream(stream: &mut Stream) -> Result<Self::Output, Error> {
        Ok(T::from_stream(stream).ok())
    }
}

impl<T> FromStream for Vec<T>
where
    T: FromStream,
{
    type Output = Vec<T::Output>;
    fn from_stream(stream: &mut Stream) -> Result<Self::Output, Error> {
        let mut vec = Vec::new();
        while let Ok(item) = T::from_stream(stream) {
            vec.push(item);
        }
        Ok(vec)
    }
}

impl<T: DiagDisplay> DiagDisplay for [T] {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for item in self {
            item.fmt(f)?;
        }

        Ok(())
    }
}

impl<T> MatchStream for [T]
where
    T: MatchStream,
{
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        let mut total = 0;
        for item in self {
            let len = item.match_stream(stream.view())?;
            total += len;
            stream.skip(len);
        }

        Ok(total)
    }
}

impl<T: DiagDisplay, const N: usize> DiagDisplay for [T; N] {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for item in self {
            item.fmt(f)?;
        }

        Ok(())
    }
}

impl<T, const N: usize> MatchStream for [T; N]
where
    T: MatchStream,
{
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        let mut total = 0;
        for item in self {
            let len = item.match_stream(stream.view())?;
            total += len;
            stream.skip(len);
        }

        Ok(total)
    }
}

impl<T: DiagDisplay> DiagDisplay for Vec<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for item in self {
            item.fmt(f)?;
        }

        Ok(())
    }
}

impl<T> MatchStream for Vec<T>
where
    T: MatchStream,
{
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        let mut total = 0;
        for item in self {
            let len = item.match_stream(stream.view())?;
            total += len;
            stream.skip(len);
        }

        Ok(total)
    }
}

macro_rules! impl_tuple {
    ($($t:ident)+) => {
        impl<$($t),+> FromStream for ($($t,)+)
        where
            $($t: FromStream),+
        {
            type Output = ($($t::Output,)+);

            fn from_stream(stream: &mut Stream) -> Result<Self::Output, Error> {
                Ok(($(
                    $t::from_stream(stream)?,
                )+))
            }
        }

        impl<$($t),+> DiagDisplay for ($($t,)+)
        where
            $($t: DiagDisplay),+
        {
            #[allow(non_snake_case)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let ($($t,)+) = self;
                $( $t.fmt(f)?; )+
                Ok(())
            }
        }

        impl<$($t),+> MatchStream for ($($t,)+)
        where
            $($t: MatchStream),+
        {
            #[allow(non_snake_case)]
            fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
            where
                S: StreamLike,
            {
                let ($($t,)+) = self;
                let mut total = 0;
                $( total += $t.match_stream(stream.view_from(total))?; )+
                Ok(total)
            }
        }
    };
}

impl_tuple!(A B C D E F G H I J K L M N O P);
impl_tuple!(A B C D E F G H I J K L M N O);
impl_tuple!(A B C D E F G H I J K L M N);
impl_tuple!(A B C D E F G H I J K L M);
impl_tuple!(A B C D E F G H I J K L);
impl_tuple!(A B C D E F G H I J K);
impl_tuple!(A B C D E F G H I J);
impl_tuple!(A B C D E F G H I);
impl_tuple!(A B C D E F G H);
impl_tuple!(A B C D E F G);
impl_tuple!(A B C D E F);
impl_tuple!(A B C D E);
impl_tuple!(A B C D);
impl_tuple!(A B C);
impl_tuple!(A B);
impl_tuple!(A);
