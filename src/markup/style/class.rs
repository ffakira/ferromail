//! HTML `class` attribute tokens.

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
