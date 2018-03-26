use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError<L, T, E> {
    /// Generated by the parser when it encounters a token (or EOF) it did not
    /// expect.
    InvalidToken { location: L },

    /// Generated by the parser when it encounters a token (or EOF) it did not
    /// expect.
    UnrecognizedToken {
        /// If this is `Some`, then an unexpected token of type `T`
        /// was observed, with a span given by the two `L` values. If
        /// this is `None`, then EOF was observed when it was not
        /// expected.
        token: Option<(L, T, L)>,

        /// The set of expected tokens: these names are taken from the
        /// grammar and hence may not necessarily be suitable for
        /// presenting to the user.
        expected: Vec<String>,
    },

    /// Generated by the parser when it encounters additional,
    /// unexpected tokens.
    ExtraToken { token: (L, T, L) },

    /// Custom error type.
    User { error: E },
}

impl<L, T, E> ParseError<L, T, E> {
    fn map_intern<FL, LL, FT, TT, FE, EE>(
        self,
        loc_op: FL,
        tok_op: FT,
        err_op: FE,
    ) -> ParseError<LL, TT, EE>
    where
        FL: Fn(L) -> LL,
        FT: Fn(T) -> TT,
        FE: Fn(E) -> EE,
    {
        let maptok = |(s, t, e): (L, T, L)| (loc_op(s), tok_op(t), loc_op(e));
        match self {
            ParseError::InvalidToken { location } => ParseError::InvalidToken {
                location: loc_op(location),
            },
            ParseError::UnrecognizedToken { token, expected } => ParseError::UnrecognizedToken {
                token: token.map(maptok),
                expected: expected,
            },
            ParseError::ExtraToken { token } => ParseError::ExtraToken {
                token: maptok(token),
            },
            ParseError::User { error } => ParseError::User {
                error: err_op(error),
            },
        }
    }

    pub fn map_location<F, LL>(self, op: F) -> ParseError<LL, T, E>
    where
        F: Fn(L) -> LL,
    {
        self.map_intern(op, |x| x, |x| x)
    }

    pub fn map_token<F, TT>(self, op: F) -> ParseError<L, TT, E>
    where
        F: Fn(T) -> TT,
    {
        self.map_intern(|x| x, op, |x| x)
    }

    pub fn map_error<F, EE>(self, op: F) -> ParseError<L, T, EE>
    where
        F: Fn(E) -> EE,
    {
        self.map_intern(|x| x, |x| x, op)
    }
}

impl<L, T, E> fmt::Display for ParseError<L, T, E>
where
    L: fmt::Display,
    T: fmt::Display,
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        match *self {
            InvalidToken { ref location } => write!(f, "Invalid token at {}", location),
            UnrecognizedToken {
                ref token,
                ref expected,
            } => {
                match *token {
                    Some((ref start, ref token, ref end)) => try!(write!(
                        f,
                        "Unrecognized token `{}` found at {}:{}",
                        token, start, end
                    )),
                    None => try!(write!(f, "Unrecognized EOF")),
                }
                if !expected.is_empty() {
                    try!(writeln!(f, ""));
                    for (i, e) in expected.iter().enumerate() {
                        let sep = match i {
                            0 => "Expected one of",
                            _ if i < expected.len() - 1 => ",",
                            // Last expected message to be written
                            _ => " or",
                        };
                        try!(write!(f, "{} {}", sep, e));
                    }
                }
                Ok(())
            }
            ExtraToken {
                token: (ref start, ref token, ref end),
            } => write!(f, "Extra token {} found at {}:{}", token, start, end),
            User { ref error } => write!(f, "{}", error),
        }
    }
}

impl<L, T, E> Error for ParseError<L, T, E>
where
    L: fmt::Debug + fmt::Display,
    T: fmt::Debug + fmt::Display,
    E: fmt::Debug + fmt::Display,
{
    fn description(&self) -> &str {
        "parse error"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ErrorRecovery<L, T, E> {
    pub error: ParseError<L, T, E>,
    pub dropped_tokens: Vec<(L, T, L)>,
}

/// Define a module using the generated parse from a `.lalrpop` file.
///
/// You have to specify the name of the module and the path of the file
/// generated by LALRPOP. If the input is in the root directory, you can
/// omit it.
///
/// # Example
/// ```ignore
/// // load parser in src/parser.lalrpop
/// lalrpop_mod!(parser);
///
/// // load parser in src/lex/parser.lalrpop
/// lalrpop_mod!(parser, "/lex/parser.rs");
/// ```

#[macro_export]
macro_rules! lalrpop_mod {
    ($modname:ident) => { lalrpop_mod!($modname, concat!("/", stringify!($modname), ".rs")); };

    ($modname:ident, $source:expr) => { mod $modname { include!(concat!(env!("OUT_DIR"), $source)); } };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let err = ParseError::UnrecognizedToken::<i32, &str, &str> {
            token: Some((1, "t0", 2)),
            expected: vec!["t1", "t2", "t3"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        };
        assert_eq!(
            format!("{}", err),
            "Unrecognized token `t0` found at 1:2\n\
             Expected one of t1, t2 or t3"
        );
    }
}
