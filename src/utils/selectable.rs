use colored::Colorize;
use std::fmt::Display;

/// The message to display when the user wants to exit the `Select` prompt.
const DESELECT_MSG: &str = "← Go Back";

/// A wrapper for a type `T` that behaves like an option.
///
/// It should be used with `inquire::Select` in particular, as it allows
/// the user to select a value from a list of values, or to go back.
///
/// ### Example
///
/// The logic is similar to using `prompt_skippable` with a `let Some(_)` pattern match.
///
/// ```rust
/// // before
/// let Some(module) = Select::new(
///     "Select something:",
///     foobar,
/// )
/// .prompt_skippable()?
/// else {
///     break;
/// };
/// ```
///
/// Here, we instead use `prompt` with a `let Selectable::Some(_)` pattern match.
///
/// ```rust
/// // after
/// let Selectable::Some(module) = Select::new(
///     "Select something:",
///     Selectable::new(foobar),
/// )
/// .prompt()?
/// else {
///     break;
/// };
/// ```
///
/// You can even pattern match to `Some(Selectable::Some(foo))` to allow both
/// skippable prompt & a go-back option.
pub enum Selectable<T> {
    Some(T),
    None,
}

impl<T> Selectable<T>
where
    T: Display,
{
    /// Create a new selectable object
    pub fn new(values: Vec<T>) -> Vec<Self> {
        values
            .into_iter()
            .map(Self::Some)
            .chain(std::iter::once(Self::None))
            .collect()
    }
}

impl<T: Display> Display for Selectable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Some(ref value) => write!(f, "{}", value),
            Self::None => write!(f, "{}", DESELECT_MSG.bold()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_selectable() {
        let values = vec!["a", "b", "c"];
        let selectable = Selectable::new(values);

        assert_eq!(selectable.len(), 4);
        assert_eq!(selectable[0].to_string(), "a");
        assert_eq!(selectable[1].to_string(), "b");
        assert_eq!(selectable[2].to_string(), "c");

        // the last one should be the deselect message
        // but we test like this to ignore color codes
        assert_eq!(selectable[3].to_string(), DESELECT_MSG.bold().to_string());
    }
}
