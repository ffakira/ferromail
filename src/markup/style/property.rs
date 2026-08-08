//! CSS property names.

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Property(String);

impl Property {
    pub fn new(raw: &str) -> Option<Self> {
        let ok = !raw.is_empty()
            && raw.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
            && !raw.starts_with('-');

        ok.then(|| Property(raw.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
