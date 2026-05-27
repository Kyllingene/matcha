use crate::{FromStream, MatchStream, StreamView, StreamLike, Error, ErrorText, DiagDisplay};
use crate::procmacro::TokenTree;

/// A piece of punctuation (e.g. `!`, `.`, `:`).
///
/// See also [`punct`] for helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Punct(pub char);

impl FromStream for Punct {
    type Output = Self;
    fn from_stream<S>(stream: &mut S) -> Result<Self, Error>
    where
        S: StreamLike,
    {
        let tok = match stream.peek() {
            Some(tt) => tt,
            None => {
                return Err(stream.err_eos(ErrorText::Plain("punctuation".into())));
            }
        };
        if let TokenTree::Punct(p) = tok {
            let c = p.as_char();
            stream.pop();
            Ok(Self(c))
        } else {
            Err(Error::new(
                ErrorText::Plain("punctuation".into()),
                ErrorText::Backticks(tok.to_string()),
                tok.span(),
            ))
        }
    }
}

impl MatchStream for Punct {
    fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
    where
        S: StreamLike,
    {
        let tok = stream.peek().ok_or(None)?;
        if let TokenTree::Punct(p) = tok {
            (p.as_char() == self.0)
                .then_some(1)
                .ok_or_else(|| Some(tok.to_string()))
        } else {
            Err(Some(tok.to_string()))
        }
    }
}

impl core::fmt::Display for Punct {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl DiagDisplay for Punct {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Punct {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        use quote::TokenStreamExt;
        stream.append(proc_macro2::Punct::new(self.0, proc_macro2::Spacing::Alone));
    }
}

/// Helpers for parsing specific [`Punct`]s.
pub mod punct {
    use crate::procmacro::TokenTree;
    use crate::{DiagDisplay, Error, ErrorText, FromStream, MatchStream, StreamLike, StreamView};

    macro_rules! impl_punct {
        ($(
            $name:ident: $p:literal = $doc:literal
        ),+ $(,)?) => {$(
            #[doc = concat!(
                "A helper for parsing ",
                $doc,
                " into a [`Punct`](crate::Punct).",
            )]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $name;

            impl FromStream for $name {
                type Output = Self;
                fn from_stream<S>(stream: &mut S) -> Result<Self, Error>
where S: StreamLike, {
                    let tok = match stream.peek() {
                        Some(tt) => tt,
                        None => {
                            return Err(stream.err_eos(ErrorText::Backticks($p.to_string())));
                        }
                    };
                    if let TokenTree::Punct(p) = tok
                        && p.as_char() == $p {
                        stream.pop();
                        Ok(Self)
                    } else {
                        Err(Error::new(ErrorText::Backticks($p.to_string()),  ErrorText::Backticks(tok.to_string()), tok.span()))
                    }
                }
            }

            impl MatchStream for $name {
                fn match_stream<S>(&self, mut stream: StreamView<'_, S>) -> Result<usize, Option<String>>
                where
                    S: StreamLike,
                {
                    let tok = stream.peek().ok_or(None)?;
                    if let TokenTree::Punct(p) = tok {
                        (p.as_char() == $p).then_some(1).ok_or_else(|| Some(tok.to_string()))
                    } else {
                        Err(Some(tok.to_string()))
                    }
                }
            }

            #[cfg(feature = "quote")]
            impl quote::ToTokens for $name {
                fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
                    use quote::TokenStreamExt;
                    stream.append(proc_macro2::Punct::new($p, proc_macro2::Spacing::Alone));
                }
            }

            impl core::fmt::Display for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    $p.fmt(f)
                }
            }

            impl DiagDisplay for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{self}")
                }
            }
        )+};
    }

    impl_punct! {
        Slash: '/' = "a slash (`/`)",
        Bar: '|' = "a pipe (`|`)",
        Colon: ':' = "a colon (`:`)",
        Semicolon: ';' = "a semicolon (`;`)",
        Quote: '"' = "a quote (`\"`)",
        Apos: '\'' = "an apostrophe (`'`)",
        Comma: ',' = "a comma (`,`)",
        Less: '<' = "a left angle bracket (`<`)",
        Period: '.' = "a period (`.`)",
        Greater: '>' = "a right angle bracket (`>`)",
        Backslash: '\\' = "a backslash (`\\`)",
        Question: '?' = "a question mark (`?`)",

        Tilde: '~' = "a tilde (`~`)",
        Backtick: '`' = "a backtick (`` ` ``)",
        Bang: '!' = "an exclamation mark (`!`)",
        At: '@' = "an at symbol (`@`)",
        Hash: '#' = "a pound sign (`#`)",
        Dollar: '$' = "a dollar sign (`$`)",
        Percent: '%' = "a percent sign (`%`)",
        Caret: '^' = "a caret (`^`)",
        Amp: '&' = "an ampersand (`&`)",
        Star: '*' = "an asterisk (`*`)",

        Dash: '-' = "a minus sign (`-`)",
        Underscore: '_' = "an underscore (`_`)",
        Plus: '+' = "a plus sign (`+`)",
        Equals: '=' = "an equals sign (`=`)",
    }
}
