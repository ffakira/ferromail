//! URL parsing behind a scheme allowlist.

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Url(String);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum UrlError {
    Empty,
    DisallowedScheme,
}

impl Url {
    /// Parses an absolute `http`, `https`, `mailto` or `tel` URL.
    ///
    /// Relative URLs are rejected, since an email has no base to resolve them against.
    ///
    /// # Errors
    ///
    /// [`UrlError::Empty`] if the input is blank, [`UrlError::DisallowedScheme`]
    /// if the scheme is missing or not one of the four above.
    ///
    /// ```
    /// # use ferromail::markup::{Url, UrlError};
    /// assert!(Url::parse("https://example.com").is_ok());
    /// assert_eq!(Url::parse("javascript:alert(1)"), Err(UrlError::DisallowedScheme));
    /// ```
    pub fn parse(raw: &str) -> Result<Self, UrlError> {
        let cleaned: String = raw.chars().filter(|c| !c.is_control()).collect();

        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            return Err(UrlError::Empty);
        }

        let scheme = trimmed
            .split_once(':')
            .map(|(s, _)| s.to_ascii_lowercase())
            .ok_or(UrlError::DisallowedScheme)?;

        // Allowlist, not blocklist: blocking javascript: invites vbscript:,
        // data:text/html, and every future variant. Control chars are stripped
        // first because a tab inside the scheme gets normalised away by clients.
        match scheme.as_str() {
            "http" | "https" | "mailto" | "tel" => Ok(Url(trimmed.to_owned())),
            _ => Err(UrlError::DisallowedScheme),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
