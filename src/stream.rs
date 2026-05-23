use crate::procmacro::{Span, TokenStream, TokenTree};
use std::collections::VecDeque;

/// The text for an [`Error`].
#[derive(Debug, Clone)]
pub enum ErrorText {
    /// Renders as `"end of stream"`.
    EndOfStream,
    /// Renders as `"token"`.
    Token,
    /// Renders without markup.
    Plain(String),
    /// Renders surrounded by backticks.
    Backticks(String),
}

impl core::fmt::Display for ErrorText {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EndOfStream => write!(f, "end of stream"),
            Self::Token => write!(f, "token"),
            Self::Plain(s) => write!(f, "{s}"),
            Self::Backticks(s) => write!(f, "`{s}`"),
        }
    }
}

/// An error that occured during parsing a [`Stream`].
#[derive(Debug, Clone)]
pub struct Error {
    /// The expected data.
    pub expected: ErrorText,

    /// The found data.
    pub got: ErrorText,

    /// The start location at which the error was encountered.
    ///
    /// If that location isn't available, you should default to
    /// [`Span::call_site`].
    ///
    /// Is not available if feature `proc-macro2` is enabled but not
    /// `proc-macro2-span-locations`.
    #[cfg(any(feature = "proc-macro2-span-locations", not(feature = "proc-macro2")))]
    pub at: Span,
}

impl Error {
    /// Create a new error.
    ///
    /// If feature `proc-macro2` is enabled but not
    /// `proc-macro2-span-locations`, ignores `at`.
    #[cfg_attr(all(not(feature = "proc-macro2-span-locations"), feature = "proc-macro2"),
        expect(unused_variables, reason = "`at` isn't used without span locations"))]
    pub const fn new(
        expected: ErrorText,
        got: ErrorText,
        at: Span,
    ) -> Self {
        Self {
            expected,
            got,
            #[cfg(any(feature = "proc-macro2-span-locations", not(feature = "proc-macro2")))]
            at,
        }
    }
}

#[cfg(feature = "proc-macro2")]
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "expected {}, got {}", self.expected, self.got)?;

        #[cfg(feature = "proc-macro2-span-locations")]
        {
            let lc = self.at.start();
            write!(f, " (at {}:{}:{})", self.at.file(), lc.line, lc.column,)?;
        }

        Ok(())
    }
}

#[cfg(not(feature = "proc-macro2"))]
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "expected {}, got {} (at {}:{}:{})",
            self.expected,
            self.got,
            self.at.file(),
            self.at.line(),
            self.at.column(),
        )
    }
}

/// Parse a type out of a [`Stream`].
pub trait FromStream {
    /// The type to be produced.
    ///
    /// Need not be `Self`; see [`crate::Parens`] for an example.
    type Output: Sized;

    /// Attempt to parse the type from the stream.
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
    fn pop(&mut self) -> Option<TokenTree>;
    fn peek(&mut self) -> Option<&TokenTree>;
    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree>;
    fn peek_last(&mut self) -> Option<&TokenTree>;
    fn peek_many(&mut self, n: usize) -> &[TokenTree];
    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree];
    fn peek_from(&mut self, at: usize) -> &[TokenTree];
    fn skip(&mut self, n: usize);

    fn stringify(&mut self) -> String;
    fn last_span(&self) -> Option<Span>;

    fn err_eos(&self, expected: ErrorText) -> Error {
        Error {
            expected,
            got: ErrorText::EndOfStream,
            #[cfg(any(feature = "proc-macro2-span-locations", not(feature = "proc-macro2")))]
            at: self.last_span().unwrap_or_else(Span::call_site),
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
        let last_span = self.last_span();
        StreamView {
            stream: self,
            skip: 0,
            last_span,
        }
    }

    /// Returns a new sub-view based on `self`, skipping the first `skip`
    /// tokens.
    fn view_from(&mut self, skip: usize) -> StreamView<'_, Self>
    where
        Self: Sized,
    {
        let last_span = self.last_span();
        StreamView {
            stream: self,
            skip,
            last_span,
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
    fn last_span(&self) -> Option<Span> {
        (**self).last_span()
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
        let last_span = self.last_span();
        StreamView {
            stream: self,
            skip: 0,
            last_span,
        }
    }
    fn view_from(&mut self, skip: usize) -> StreamView<'_, Self>
    where
        Self: Sized,
    {
        let last_span = self.last_span();
        StreamView {
            stream: self,
            skip,
            last_span,
        }
    }
}

/// A buffered stream from a [`TokenStream`].
pub struct Stream {
    iter: crate::procmacro::token_stream::IntoIter,
    buffer: VecDeque<TokenTree>,
    last_span: Option<Span>,
}

impl From<TokenStream> for Stream {
    fn from(ts: TokenStream) -> Self {
        Self {
            iter: ts.into_iter(),
            buffer: VecDeque::new(),
            last_span: None,
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
}

impl StreamLike for Stream {
    /// Returns the next token in the stream, if any.
    fn peek(&mut self) -> Option<&TokenTree> {
        self.peek_nth(0)
    }

    /// Returns the `n`th token in the stream, if any.
    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        if self.buffer.len() <= n {
            self.pull(n + 1);
        }

        self.buffer.get(n)
    }

    /// Returns the last token in the stream, if any.
    fn peek_last(&mut self) -> Option<&TokenTree> {
        self.pull(usize::MAX);
        self.buffer.back()
    }

    /// Returns the next `n` tokens in the stream, if any.
    fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        self.peek_many_at(n, 0)
    }

    /// Returns the next `n` tokens in the stream, starting at `at`, if any.
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

    /// Returns the last tokens in the stream, starting at `n`, if any.
    fn peek_from(&mut self, at: usize) -> &[TokenTree] {
        self.pull(usize::MAX);
        self.buffer.make_contiguous();

        let start = self.contiguous_len().min(at);
        &self.buffer.as_slices().0[start..]
    }

    /// Consumes the next token in the stream, if any.
    ///
    /// Also updates [`Self::last_span`].
    fn pop(&mut self) -> Option<TokenTree> {
        if let Some(tt) = self.buffer.pop_front() {
            self.last_span = Some(tt.span());
            Some(tt)
        } else {
            self.iter.next()
        }
    }

    /// Skips the next `n` tokens.
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

    /// Stringifies the remaining tokens in the stream.
    ///
    /// Note that it's impossible to get the span or source for a `TokenStream`,
    /// so this is lossy; it simply concatenates all the remaining tokens with a
    /// space between each one. This is purely for diagnostic purposes.
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

    fn last_span(&self) -> Option<Span> {
        self.last_span
    }

    /// Creates a new [`Error`] with <code>got: [ErrorText::EndOfStream]</code>.
    ///
    ///
    /// `at` is set the <code>self.[last_span](Self::last_span)()</code>.
    /// If `self.last_span()` is `None`, defaults to [`Span::call_site`].
    ///
    /// `at` is not set if feature `proc-macro2` is enabled but not
    /// `proc-macro2-span-locations`.
    fn err_eos(&self, expected: ErrorText) -> Error {
        Error {
            expected,
            got: ErrorText::EndOfStream,
            #[cfg(any(feature = "proc-macro2-span-locations", not(feature = "proc-macro2")))]
            at: self.last_span().unwrap_or_else(Span::call_site),
        }
    }
}

/// A view into a [`Stream`] that's allowed to pull into its buffer, but not
/// allowed to pop elements.
pub struct StreamView<'a, S> {
    stream: &'a mut S,
    skip: usize,
    last_span: Option<Span>,
}

impl<S: StreamLike> StreamView<'_, S> {
    /// Returns how many tokens have been skipped so far.
    pub fn skipped(&self) -> usize {
        self.skip
    }

    /// Un-skips any tokens that have been skipped.
    ///
    /// See also [`Self::reset_skip_to`].
    pub fn reset_skip(&mut self) {
        self.skip = 0;
    }
}

impl<S: StreamLike> StreamLike for StreamView<'_, S> {
    fn pop(&mut self) -> Option<TokenTree> {
        let tt = self.peek()?.clone();
        self.skip(1);
        self.last_span = Some(tt.span());
        Some(tt)
    }

    /// Skips `n` tokens.
    fn skip(&mut self, n: usize) {
        self.skip += n;
    }

    /// See [`Stream::peek`].
    fn peek(&mut self) -> Option<&TokenTree> {
        self.stream.peek_nth(self.skip)
    }

    /// See [`Stream::peek_nth`].
    fn peek_nth(&mut self, n: usize) -> Option<&TokenTree> {
        self.stream.peek_nth(n + self.skip)
    }

    /// See [`Stream::peek_last`].
    fn peek_last(&mut self) -> Option<&TokenTree> {
        self.stream.peek_last()
    }

    /// See [`Stream::peek_many`].
    fn peek_many(&mut self, n: usize) -> &[TokenTree] {
        self.stream.peek_many_at(n, self.skip)
    }

    /// See [`Stream::peek_many_at`].
    fn peek_many_at(&mut self, n: usize, at: usize) -> &[TokenTree] {
        self.stream.peek_many_at(n, at + self.skip)
    }

    /// See [`Stream::peek_from`].
    fn peek_from(&mut self, at: usize) -> &[TokenTree] {
        self.stream.peek_from(at)
    }

    /// See [`Stream::stringify`].
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

    fn last_span(&self) -> Option<Span> {
        self.last_span
    }
}
