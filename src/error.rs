use crate::procmacro::{Span, TokenStream};

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

/// An error that occured during parsing a [`Stream`](crate::Stream).
#[derive(Debug, Clone)]
pub struct Error {
    /// The expected data.
    pub expected: ErrorText,

    /// The found data.
    pub got: ErrorText,

    /// Whether or not the error is recoverable.
    ///
    /// If `true`, you *must* bubble it up. You are not allowed to try another
    /// parsing path.
    pub fatal: bool,

    /// The start location at which the error was encountered.
    ///
    /// If that location isn't available, you should default to
    /// [`Span::call_site`].
    pub at: Span,
}

impl Error {
    /// Create a new error.
    pub const fn new(expected: ErrorText, got: ErrorText, at: Span) -> Self {
        Self {
            expected,
            got,
            fatal: false,
            at,
        }
    }

    /// Generates a compile error with the enclosed information.
    pub fn throw(&self) -> TokenStream {
        crate::error(format!("{self}"), self.at)
    }

    /// Flags this error as fatal.
    ///
    /// Fatal errors must not be recovered from. If you encounter a fatal error,
    /// you must return it untouched.
    ///
    /// See also [`with_fatal`](Self::with_fatal).
    pub const fn fatal(mut self) -> Self {
        self.fatal = true;
        self
    }

    /// Flags this error as fatal or not fatal.
    ///
    /// Fatal errors must not be recovered from. If you encounter a fatal error,
    /// you must return it untouched.
    ///
    /// See also the shorthand [`fatal`](Self::fatal).
    pub const fn with_fatal(mut self, fatal: bool) -> Self {
        self.fatal = fatal;
        self
    }
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "expected {}, got {}", self.expected, self.got,)
    }
}
