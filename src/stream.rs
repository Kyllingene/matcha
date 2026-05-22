use proc_macro::{TokenStream, TokenTree, Span};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum Error {
    EndOfStream,
    Invalid(Span),

    Expected {
        expected: Option<String>,
        got: Option<String>,
        at: Span,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EndOfStream => write!(f, "unexpected end of input"),
            Self::Invalid(span) => write!(
                f,
                "invalid input (at {}:{}:{})",
                span.local_file().unwrap_or_else(|| "<...>".into()).display(),
                span.line(),
                span.column(),
            ),
            Self::Expected { expected, got, at } => {
                if let Some(s) = expected {
                    write!(f, "expected `{s}`, got")?;
                } else {
                    write!(f, "expected end of stream, got")?;
                }

                if let Some(s) = got {
                    write!(f, "`{s}`")?;
                } else {
                    write!(f, "end of stream")?;
                }

                write!(
                    f,
                    "(at {}:{}:{})",
                    at.local_file().unwrap_or_else(|| "<...>".into()).display(),
                    at.line(),
                    at.column(),
                )
            }
        }
    }
}

pub trait FromStream {
    type Output: Sized;

    fn from_stream(stream: &mut Stream) -> Result<Self::Output, Error>;
}

pub trait MatchStream {
    fn match_stream<S>(&self, stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike;
}

pub trait StreamLike {
    fn peek(&mut self) -> Option<&TokenTree>;
    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree>;
    fn peek_last(&mut self) -> Option<&TokenTree>;
    fn peek_many(&mut self, n: usize) -> &[TokenTree];
    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree];

    fn stringify(&mut self) -> String;

    fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }

    fn view(&mut self) -> StreamView<'_, Self>
    where Self: Sized,
    {
        StreamView {
            stream: self,
            skip: 0,
        }
    }

    fn view_from(&mut self, skip: usize) -> StreamView<'_, Self>
    where Self: Sized,
    {
        StreamView { stream: self, skip }
    }
}

pub struct Stream {
    iter: proc_macro::token_stream::IntoIter,
    buffer: VecDeque<TokenTree>,
}

impl From<TokenStream> for Stream {
    fn from(ts: TokenStream) -> Self {
        Self {
            iter: ts.into_iter(),
            buffer: VecDeque::new(),
        }
    }
}

impl Stream {
    fn pull(&mut self, n: usize) {
        if self.buffer.len() >= n {
            return;
        }

        self.buffer.extend((&mut self.iter).take(n));
    }

    fn contiguous_len(&self) -> usize {
        self.buffer.as_slices().0.len()
    }

    pub fn peek(&mut self) -> Option<&TokenTree> {
        self.peek_nth(0)
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        if self.buffer.len() <= n {
            self.pull(n + 1);
        }

        self.buffer.get(n)
    }

    pub fn peek_last(&mut self) -> Option<&TokenTree> {
        self.pull(usize::MAX);
        self.buffer.back()
    }

    pub fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        self.peek_many_at(n, 0)
    }

    pub fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree] {
        if self.buffer.len() < (n + at) {
            self.pull(n + at);
        }

        if self.contiguous_len() < (n + at) {
            self.buffer.make_contiguous();
        }

        let end = self.contiguous_len().min(n);
        let start = at.min(end);
        &self.buffer.as_slices().0[start..][..end]
    }

    pub fn pop(&mut self) -> Option<TokenTree> {
        if let Some(tt) = self.buffer.pop_front() {
            Some(tt)
        } else {
            self.iter.next()
        }
    }

    pub fn skip(&mut self, n: usize) {
        if self.buffer.len() < n {
            let rest = n - self.buffer.len();
            self.buffer.drain(..);
            for _ in 0..rest { self.iter.next(); }
        } else {
            self.buffer.drain(..n);
        }
    }

    pub fn stringify(&mut self) -> String {
        self.pull(usize::MAX);

        let mut s = String::new();
        for tt in &self.buffer {
            if !s.is_empty() { s.push(' '); }
            s.push_str(&tt.to_string());
        }
        s
    }

    pub fn view(&mut self) -> StreamView<'_, Self> {
        StreamView {
            stream: self,
            skip: 0,
        }
    }

    pub fn view_from(&mut self, skip: usize) -> StreamView<'_, Self> {
        StreamView { stream: self, skip }
    }
}

/// A view into a [`Stream`] that's allowed to pull into its buffer, but not
/// allowed to pop elements.
pub struct StreamView<'a, S> {
    stream: &'a mut S,
    skip: usize,
}

impl<S: StreamLike> StreamView<'_, S> {
    pub fn skipped(&self) -> usize {
        self.skip
    }

    pub fn reset_skip(&mut self) {
        self.reset_skip_to(0);
    }

    pub fn reset_skip_to(&mut self, n: usize) {
        self.skip = n;
    }

    pub fn skip(&mut self, n: usize) {
        self.skip += n;
    }

    pub fn peek(&mut self) -> Option<&TokenTree> {
        self.stream.peek_nth(self.skip)
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        self.stream.peek_nth(n + self.skip)
    }

    pub fn peek_last(&mut self) -> Option<&TokenTree> {
        self.stream.peek_last()
    }

    pub fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        self.stream.peek_many_at(n, self.skip)
    }

    pub fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree] {
        self.stream.peek_many_at(n, at + self.skip)
    }

    pub fn view(&mut self) -> StreamView<'_, Self> {
        StreamView {
            stream: self,
            skip: 0,
        }
    }

    pub fn stringify(&mut self) -> String {
        self.stream.stringify()
    }

    pub fn view_from(&mut self, skip: usize) -> StreamView<'_, Self> {
        StreamView { stream: self, skip }
    }
}

impl StreamLike for Stream {
    fn peek(&mut self) -> Option<&TokenTree> {
        self.peek()
    }
    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        self.peek_nth(n)
    }
    fn peek_last(&mut self) -> Option<&TokenTree> {
        self.peek_last()
    }
    fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        self.peek_many(n)
    }
    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree] {
        self.peek_many_at(n, at)
    }
    fn stringify(&mut self) -> String {
        self.stringify()
    }
}

impl<S: StreamLike> StreamLike for StreamView<'_, S> {
    fn peek(&mut self) -> Option<&TokenTree> {
        self.peek()
    }
    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        self.peek_nth(n)
    }
    fn peek_last(&mut self) -> Option<&TokenTree> {
        self.peek_last()
    }
    fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        self.peek_many(n)
    }
    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree] {
        self.peek_many_at(n, at)
    }
    fn stringify(&mut self) -> String {
        self.stringify()
    }
}
