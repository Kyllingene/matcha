use crate::procmacro::Span;

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

    /// Whether or not the error is recoverable.
    ///
    /// If `true`, you *must* bubble it up. You are not allowed to try another
    /// parsing path.
    pub fatal: bool,

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
    #[cfg_attr(
        all(not(feature = "proc-macro2-span-locations"), feature = "proc-macro2"),
        expect(unused_variables, reason = "`at` isn't used without span locations")
    )]
    pub const fn new(expected: ErrorText, got: ErrorText, at: Span) -> Self {
        Self {
            expected,
            got,
            fatal: false,
            #[cfg(any(feature = "proc-macro2-span-locations", not(feature = "proc-macro2")))]
            at,
        }
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
