//! Declaration values that cannot escape a `style` attribute.

use crate::markup::Url;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StyleValue(String);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum StyleValueError {
    Empty,
    IllegalChar(char),
    Comment,
}

impl StyleValue {
    /// Parses a single CSS declaration value.
    ///
    /// Functions are not accepted. Legacy clients want hex colours, and
    /// `url()` has its own constructor.
    ///
    /// # Errors
    /// [`StyleValueError::Empty`] if blank, [`StyleValueError::IllegalChar`]
    /// for anything that could end the declaration or the attribute, and
    /// [`StyleValueError::Comment`] for a CSS comment.
    pub fn parse(raw: &str) -> Result<Self, StyleValueError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(StyleValueError::Empty);
        }

        for c in trimmed.chars() {
            if c.is_control()
                || matches!(
                    c,
                    ';' | '"' | '<' | '>' | '&' | '\\' | '(' | ')' | '{' | '}' | '@'
                )
            {
                return Err(StyleValueError::IllegalChar(c));
            }
        }

        if trimmed.contains("/*") {
            return Err(StyleValueError::Comment);
        }

        Ok(StyleValue(trimmed.to_owned()))
    }

    /// Builds a `url(...)` value from an already-validated [`Url`].
    ///
    /// Infallible: [`Url::parse`] has already applied the scheme allowlist.
    /// Single quotes are used because the declaration is rendered inside a
    /// double-quoted `style` attribute; anything that could close the string,
    /// the function, the declaration or the attribute is percent-encoded.
    pub fn url(url: &Url) -> Self {
        let mut out = String::from("url('");

        for c in url.as_str().chars() {
            match c {
                '\'' => out.push_str("%27"),
                '"' => out.push_str("%22"),
                '(' => out.push_str("%28"),
                ')' => out.push_str("%29"),
                '\\' => out.push_str("%5C"),
                ';' => out.push_str("%3B"),
                '<' => out.push_str("%3C"),
                '>' => out.push_str("%3E"),
                '&' => out.push_str("%26"),
                ' ' => out.push_str("%20"),
                _ => out.push(c),
            }
        }

        out.push_str("')");
        StyleValue(out)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_values() {
        for v in [
            "#333",
            "#ff8800",
            "14px",
            "Arial, sans-serif",
            "0 auto",
            "12px/1.5",
        ] {
            assert!(StyleValue::parse(v).is_ok(), "rejected {v}");
        }
    }

    #[test]
    fn rejects_every_function() {
        for v in [
            "expression(alert(1))",
            "url(javascript:alert(1))",
            "calc(100% - 10px)",
            "rgba(0,0,0,.5)",
        ] {
            assert_eq!(
                StyleValue::parse(v),
                Err(StyleValueError::IllegalChar('(')),
                "accepted {v}"
            );
        }
    }
}
