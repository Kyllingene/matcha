use crate::{Error, FromStream, MatchStream, Stream, StreamLike, StreamView};
use proc_macro::TokenTree;

impl FromStream for TokenTree {
    type Output = Self;
    fn from_stream(stream: &mut Stream) -> Result<Self, Error> {
        stream.pop().ok_or(Error::EndOfStream)
    }
}

impl MatchStream for TokenTree {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, String>
    where
        S: StreamLike,
    {
        match (self, stream.peek().ok_or_else(|| "<end of input>".to_string())?) {
            (TokenTree::Group(lhs), TokenTree::Group(rhs)) => {
                (lhs.to_string() == rhs.to_string())
                    .then_some(1)
                    .ok_or_else(|| rhs.to_string())
            }
            (TokenTree::Ident(lhs), TokenTree::Ident(rhs)) => {
                (lhs.to_string() == rhs.to_string())
                    .then_some(1)
                    .ok_or_else(|| rhs.to_string())
            }
            (TokenTree::Punct(lhs), TokenTree::Punct(rhs)) => {
                (lhs.to_string() == rhs.to_string())
                    .then_some(1)
                    .ok_or_else(|| rhs.to_string())
            }
            (TokenTree::Literal(lhs), TokenTree::Literal(rhs)) => {
                (lhs.to_string() == rhs.to_string())
                    .then_some(1)
                    .ok_or_else(|| rhs.to_string())
            }
            (_, rhs) => Err(rhs.to_string()),
        }
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

impl<T> MatchStream for Vec<T>
where
    T: MatchStream,
{
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, String>
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
