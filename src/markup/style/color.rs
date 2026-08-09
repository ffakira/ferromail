//! Validated colours.
//!
//! Its own file because a utility palette (`blue-600` and friends) will be
//! built here, and that is a lot of data next to a small type.

use super::StyleValue;

/// A validated hex colour, e.g. `#fff` or `#2563eb`.
///
/// Every character it can hold is legal in a declaration value, so
/// [`Color::style_value`] is infallible. Validating here rather than at
/// render time means a bad colour is rejected where it was written.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Color(String);

impl Color {
    /// Parses `#rgb` or `#rrggbb`, case-insensitively.
    pub fn hex(raw: &str) -> Option<Self> {
        let body = raw.trim().strip_prefix('#')?;

        if !matches!(body.len(), 3 | 6) || !body.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }

        Some(Color(format!("#{}", body.to_ascii_lowercase())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// # Panics
    ///
    /// Never in practice: `Color::hex` is the only constructor, and `#` plus
    /// hex digits are all outside `StyleValue`'s reject set.
    pub fn style_value(&self) -> StyleValue {
        StyleValue::parse(&self.0).expect("hex colour is a legal declaration value")
    }
}
