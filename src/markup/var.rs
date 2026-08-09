//! Named holes in a tree, filled at render time.

/// The name of a placeholder.
///
/// Validated so a name can be printed back into the output when a tree is
/// rendered unfilled, without that being a way to inject anything.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct VarName(String);

impl VarName {
    /// ASCII letters, digits and underscores, and not empty.
    pub fn new(raw: &str) -> Option<Self> {
        let ok = !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

        ok.then(|| VarName(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
