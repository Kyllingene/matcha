use crate::procmacro::{Span, TokenStream, TokenTree};
use crate::{Error, ErrorText};
use std::collections::VecDeque;

/// Parse a type out of a [`Stream`].
pub trait FromStream {
    /// The type to be produced.
    ///
    /// Need not be `Self`; see [`crate::Parens`] for an example.
    type Output: Sized;

    /// Attempt to parse the type from the stream.
    ///
    /// Note that you must respect [`Error::fatal`] from children. If a fatal
    /// error is encountered at any point, it must be bubbled up untouched. It
    /// is a logic error to recover from a fatal error, and will likely break
    /// downstream and/or upstream code.
    fn from_stream<S>(stream: &mut S) -> Result<Self::Output, Error>
    where
        S: StreamLike;
}

/// Test a pattern against a [`StreamView`].
pub trait MatchStream: DiagDisplay {
    /// Test the pattern against the stream.
    ///
    /// If it matches, returns `Ok(n)`, where `n` is how many tokens the pattern
    /// matched against.
    ///
    /// If it fails, returns `Err(got)`. If `got` is `Some`, then it is the
    /// data that was found instead of itself; else, the end of the stream was
    /// found before the end of the pattern.
    fn match_stream<S>(&self, stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike;
}

/// Print a [`MatchStream`] pattern out, for use in diagnostics.
///
/// Essentially just [`core::fmt::Display`].
pub trait DiagDisplay {
    /// See [`core::fmt::Display::fmt`].
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    /// See [`ToString::to_string`].
    fn diag_string(&self) -> String {
        use core::fmt::Write;

        struct Wrap<T: ?Sized>(T);
        impl<T: DiagDisplay> core::fmt::Display for Wrap<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }

        let mut s = String::new();
        write!(&mut s, "{}", Wrap(self)).expect("failed to stringify");
        s
    }
}

impl<T: DiagDisplay + ?Sized> DiagDisplay for &T {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        (**self).fmt(f)
    }
}

/// A type that can be viewed like a [`Stream`], but without popping tokens.
///
/// See [`StreamView`].
#[allow(missing_docs)]
pub trait StreamLike {
    /// Consumes the next token in the stream, if any.
    ///
    /// Also updates [`Self::span_close`].
    fn pop(&mut self) -> Option<TokenTree>;
    /// Returns the next token in the stream, if any.
    fn peek(&mut self) -> Option<&TokenTree>;
    /// Returns the `n`th token in the stream, if any.
    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree>;
    /// Returns the last token in the stream, if any.
    fn peek_last(&mut self) -> Option<&TokenTree>;
    /// Returns the next `n` tokens in the stream, if any.
    fn peek_many(&mut self, n: usize) -> &[TokenTree];
    /// Returns the next `n` tokens in the stream, starting at `at`, if any.
    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree];
    /// Returns the last tokens in the stream, starting at `n`, if any.
    fn peek_from(&mut self, at: usize) -> &[TokenTree];
    /// Skips the next `n` tokens.
    fn skip(&mut self, n: usize);

    /// Stringifies the remaining tokens in the stream.
    ///
    /// Note that it's impossible to get the span or source for a `TokenStream`,
    /// so this is lossy; it simply concatenates all the remaining tokens with a
    /// space between each one. This is purely for diagnostic purposes.
    fn stringify(&mut self) -> String;

    /// The span of the closing delimiter for this group.
    ///
    /// If there is no closing delimiter, points to the last token in the stream.
    ///
    /// ```txt
    /// `( ... )`
    ///        ^
    ///
    /// `let x = y;`
    ///           ^
    /// ```
    fn span_close(&self) -> Option<Span>;

    /// Creates a new [`Error`] with <code>got: [ErrorText::EndOfStream]</code>.
    ///
    /// `at` is set the <code>self.[span_close](Self::span_close)()</code>.
    /// If `self.span_close()` is `None`, defaults to [`Span::call_site`].
    fn err_eos(&self, expected: ErrorText) -> Error {
        Error {
            expected,
            got: ErrorText::EndOfStream,
            fatal: false,
            at: self.span_close().unwrap_or_else(Span::call_site),
        }
    }

    /// Returns whether or not there are any more tokens in the stream.
    fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Returns a new sub-view based on `self`.
    fn view(&mut self) -> StreamView<'_, Self>
    where
        Self: Sized,
    {
        let span_close = self.span_close();
        StreamView {
            stream: self,
            skip: 0,
            span_close,
        }
    }

    /// Returns a new sub-view based on `self`, skipping the first `skip`
    /// tokens.
    fn view_from(&mut self, skip: usize) -> StreamView<'_, Self>
    where
        Self: Sized,
    {
        let span_close = self.span_close();
        StreamView {
            stream: self,
            skip,
            span_close,
        }
    }
}

impl<S: StreamLike + ?Sized> StreamLike for &mut S {
    fn pop(&mut self) -> Option<TokenTree> {
        (**self).pop()
    }
    fn peek(&mut self) -> Option<&TokenTree> {
        (**self).peek()
    }
    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        (**self).peek_nth(n)
    }
    fn peek_last(&mut self) -> Option<&TokenTree> {
        (**self).peek_last()
    }
    fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        (**self).peek_many(n)
    }
    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree] {
        (**self).peek_many_at(n, at)
    }
    fn peek_from(&mut self, at: usize) -> &[TokenTree] {
        (**self).peek_from(at)
    }
    fn skip(&mut self, n: usize) {
        (**self).skip(n)
    }
    fn stringify(&mut self) -> String {
        (**self).stringify()
    }
    fn span_close(&self) -> Option<Span> {
        (**self).span_close()
    }
    fn err_eos(&self, expected: ErrorText) -> Error {
        (**self).err_eos(expected)
    }
    fn is_empty(&mut self) -> bool {
        (**self).is_empty()
    }
    fn view(&mut self) -> StreamView<'_, Self>
    where
        Self: Sized,
    {
        let span_close = self.span_close();
        StreamView {
            stream: self,
            skip: 0,
            span_close,
        }
    }
    fn view_from(&mut self, skip: usize) -> StreamView<'_, Self>
    where
        Self: Sized,
    {
        let span_close = self.span_close();
        StreamView {
            stream: self,
            skip,
            span_close,
        }
    }
}

/// A buffered stream from a [`TokenStream`].
///
/// Do not do <code>Stream::from([group](crate::Group).inner)</code>; instead,
/// do `Stream::from(group)` directly. This preserves the [closing
/// span](StreamLike::span_close) of the group, which improves diagnostics.
pub struct Stream {
    iter: crate::procmacro::token_stream::IntoIter,
    buffer: VecDeque<TokenTree>,
    span_close: Option<Span>,
}

impl From<TokenStream> for Stream {
    fn from(ts: TokenStream) -> Self {
        Self {
            iter: ts.into_iter(),
            buffer: VecDeque::new(),
            span_close: None,
        }
    }
}

impl From<crate::Group> for Stream {
    /// Creates a stream from this group's inner token stream, preserving its
    /// [closing span](StreamLike::span_close).
    fn from(group: crate::Group) -> Self {
        Self {
            iter: group.inner.into_iter(),
            buffer: VecDeque::new(),
            span_close: Some(group.span_close),
        }
    }
}

impl From<crate::procmacro::Group> for Stream {
    /// Creates a stream from this group's inner token stream, preserving its
    /// [closing span](StreamLike::span_close).
    fn from(group: crate::procmacro::Group) -> Self {
        Self {
            iter: group.stream().into_iter(),
            buffer: VecDeque::new(),
            span_close: Some(group.span_close()),
        }
    }
}

impl Stream {
    /// Overrides this stream's [closing span](StreamLike::span_close).
    ///
    /// Prefer to use `<Stream as From<Group>>::from` instead.
    pub fn set_span_close(&mut self, span_close: Span) {
        self.span_close = Some(span_close);
    }

    fn pull(&mut self, n: usize) {
        if self.buffer.len() >= n {
            return;
        }

        self.buffer.extend((&mut self.iter).take(n));
    }

    fn contiguous_len(&self) -> usize {
        self.buffer.as_slices().0.len()
    }
}

impl StreamLike for Stream {
    fn peek(&mut self) -> Option<&TokenTree> {
        self.peek_nth(0)
    }

    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        if self.buffer.len() <= n {
            self.pull(n + 1);
        }

        self.buffer.get(n)
    }

    fn peek_last(&mut self) -> Option<&TokenTree> {
        self.pull(usize::MAX);
        self.buffer.back()
    }

    fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        self.peek_many_at(n, 0)
    }

    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree] {
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

    fn peek_from(&mut self, at: usize) -> &[TokenTree] {
        self.pull(usize::MAX);
        self.buffer.make_contiguous();

        let start = self.contiguous_len().min(at);
        &self.buffer.as_slices().0[start..]
    }

    fn pop(&mut self) -> Option<TokenTree> {
        if let Some(tt) = self.buffer.pop_front() {
            if self.span_close.is_none() {
                self.span_close = Some(tt.span());
            }
            Some(tt)
        } else {
            self.iter.next()
        }
    }

    fn skip(&mut self, n: usize) {
        if self.buffer.len() < n {
            let rest = n - self.buffer.len();
            self.buffer.drain(..);
            for _ in 0..rest {
                self.iter.next();
            }
        } else {
            self.buffer.drain(..n);
        }
    }

    fn stringify(&mut self) -> String {
        self.pull(usize::MAX);

        let mut s = String::new();
        for tt in &self.buffer {
            if !s.is_empty() {
                s.push(' ');
            }
            s.push_str(&tt.to_string());
        }
        s
    }

    fn span_close(&self) -> Option<Span> {
        self.span_close
    }
}

/// A view into a [`Stream`] that's allowed to pull into its buffer, but not
/// allowed to pop elements.
pub struct StreamView<'a, S> {
    stream: &'a mut S,
    skip: usize,
    span_close: Option<Span>,
}

impl<S: StreamLike> StreamView<'_, S> {
    /// Returns how many tokens have been skipped so far.
    pub fn skipped(&self) -> usize {
        self.skip
    }

    /// Un-skips any tokens that have been skipped.
    pub fn reset_skip(&mut self) {
        self.skip = 0;
    }
}

impl<S: StreamLike> StreamLike for StreamView<'_, S> {
    fn pop(&mut self) -> Option<TokenTree> {
        let tt = self.peek()?.clone();
        if self.span_close.is_none() {
            self.span_close = Some(tt.span());
        }
        self.skip(1);
        Some(tt)
    }

    fn skip(&mut self, n: usize) {
        self.skip += n;
    }

    fn peek(&mut self) -> Option<&TokenTree> {
        self.stream.peek_nth(self.skip)
    }

    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        self.stream.peek_nth(n + self.skip)
    }

    fn peek_last(&mut self) -> Option<&TokenTree> {
        self.stream.peek_last()
    }

    fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        self.stream.peek_many_at(n, self.skip)
    }

    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree] {
        self.stream.peek_many_at(n, at + self.skip)
    }

    fn peek_from(&mut self, at: usize) -> &[TokenTree] {
        self.stream.peek_from(at + self.skip)
    }

    fn stringify(&mut self) -> String {
        let mut s = String::new();
        for tt in self.peek_from(self.skip) {
            if !s.is_empty() {
                s.push(' ');
            }
            s.push_str(&tt.to_string());
        }
        s
    }

    fn span_close(&self) -> Option<Span> {
        self.span_close
    }
}
