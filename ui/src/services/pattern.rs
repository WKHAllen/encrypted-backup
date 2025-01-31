//! Glob pattern utilities.

use glob::{Pattern, PatternError};
use std::borrow::Cow;
use std::rc::Rc;

/// Parses a glob pattern.
pub fn parse_pattern<'a, S>(pattern: S) -> Result<Pattern, (String, Rc<PatternError>)>
where
    S: Into<Cow<'a, str>>,
{
    let pattern = pattern.into();
    Pattern::new(pattern.as_ref()).map_err(|err| (pattern.into_owned(), Rc::new(err)))
}
