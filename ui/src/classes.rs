//! Utilities for creating and combining CSS classes.

use std::fmt::Display;
use std::vec::IntoIter;

/// A collection of CSS classes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Classes(Vec<String>);

impl Classes {
    /// Creates a new empty collection of CSS classes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a CSS class to the collection.
    pub fn add<C>(&mut self, class: C)
    where
        C: Into<Self>,
    {
        self.0.extend(class.into());
    }

    /// Converts the collection into a string containing all classes.
    pub fn into_classes(self) -> String {
        self.0.join(" ")
    }
}

impl Display for Classes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.join(" "))
    }
}

impl IntoIterator for Classes {
    type Item = String;
    type IntoIter = IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<String> for Classes {
    fn from(value: String) -> Self {
        Self(vec![value])
    }
}

impl From<&str> for Classes {
    fn from(value: &str) -> Self {
        Self::from(value.to_owned())
    }
}

impl<C> From<Vec<C>> for Classes
where
    C: Into<Self>,
{
    fn from(value: Vec<C>) -> Self {
        Self(value.into_iter().fold(Vec::new(), |mut acc, value| {
            acc.extend(value.into());
            acc
        }))
    }
}

impl<C> From<&[C]> for Classes
where
    C: Into<Self> + Clone,
{
    fn from(value: &[C]) -> Self {
        Self::from(value.to_vec())
    }
}

impl<C> From<&mut [C]> for Classes
where
    C: Into<Self> + Clone,
{
    fn from(value: &mut [C]) -> Self {
        Self::from(value.to_vec())
    }
}

impl<C> From<Option<C>> for Classes
where
    C: Into<Self>,
{
    fn from(value: Option<C>) -> Self {
        match value {
            Some(inner) => inner.into(),
            None => Self::new(),
        }
    }
}

/// Builds a collection of CSS classes.
macro_rules! classes {
    ( $( $class:expr ),* $(,)? ) => {{
        let mut class = Classes::new();
        $(
            class.add( $class );
        )*
        class.into_classes()
    }};
}

pub(crate) use classes;
