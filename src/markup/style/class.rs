//! HTML `class` attribute tokens.

/// A single token for the `class` attribute.
///
/// Rejects whitespace, control characters, quotes and angle brackets, so a
/// class cannot close the attribute it sits in. That is why the renderer
/// writes class names without escaping them.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClassName(String);

impl ClassName {
    pub fn new(raw: &str) -> Option<Self> {
        let ok = !raw.is_empty()
            && !raw.chars().any(|c| c.is_whitespace() || c.is_control())
            && !raw.contains(['"', '\'', '<', '>', '&']);

        ok.then(|| ClassName(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
