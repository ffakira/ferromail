use std::borrow::Cow;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RawHtml(String);

impl RawHtml {
    /// Wraps markup that is rendered verbatim, with no escaping.
    ///
    /// The caller guarantees the string is well-formed HTML from a trusted
    /// source. Passing user input here is an injection vulnerability, and
    /// this is the only place in the crate where that is possible.
    pub fn trusted(html: impl Into<String>) -> Self {
        RawHtml(html.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Node {
    Element(Element),
    Text(String),
    Raw(RawHtml),
    Conditional {
        cond: Condition,
        children: Vec<Node>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Condition {
    Mso,
    NotMso,
    MsoGte(u8),
}

impl Condition {
    pub fn expr(self) -> Cow<'static, str> {
        match self {
            Condition::Mso => Cow::Borrowed("mso"),
            Condition::NotMso => Cow::Borrowed("!mso"),
            Condition::MsoGte(version) => Cow::Owned(format!("gte mso {version}")),
        }
    }

    pub fn is_revealed(self) -> bool {
        matches!(self, Condition::NotMso)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Element {
    tag: Tag,
    attrs: Vec<(AttrName, AttrValue)>,
    urls: Vec<(UrlAttr, Url)>,
    class: Vec<ClassName>,
    styles: StyleMap,
    children: Vec<Node>,
}

impl Element {
    pub fn new(tag: Tag) -> Self {
        Self {
            tag,
            attrs: Vec::new(),
            urls: Vec::new(),
            class: Vec::new(),
            styles: StyleMap::new(),
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn attr(mut self, name: AttrName, value: AttrValue) -> Self {
        self.attrs.retain(|(n, _)| *n != name);
        self.attrs.push((name, value));
        self
    }

    /// Sets a URL-bearing attribute.
    ///
    /// [`UrlAttr`] and [`AttrName`] are disjoint, so the only way to reach
    /// `href` or `src` is through an already-parsed [`Url`]. A raw string
    /// cannot be routed here.
    #[must_use]
    pub fn url_attr(mut self, name: UrlAttr, url: Url) -> Self {
        self.urls.retain(|(n, _)| *n != name);
        self.urls.push((name, url));
        self
    }

    #[must_use]
    pub fn child(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }

    #[must_use]
    pub fn text(self, text: impl Into<String>) -> Self {
        self.child(Node::Text(text.into()))
    }

    pub fn tag(&self) -> Tag {
        self.tag
    }

    pub fn attrs(&self) -> &[(AttrName, AttrValue)] {
        &self.attrs
    }

    pub fn urls(&self) -> &[(UrlAttr, Url)] {
        &self.urls
    }

    pub fn classes(&self) -> &[ClassName] {
        &self.class
    }

    #[must_use]
    pub fn class(mut self, name: ClassName) -> Self {
        self.class.push(name);
        self
    }

    pub fn styles(&self) -> &StyleMap {
        &self.styles
    }

    #[must_use]
    pub fn style(mut self, prop: Property, value: StyleValue) -> Self {
        self.styles.set(prop, value);
        self
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }
}

impl From<Element> for Node {
    fn from(el: Element) -> Self {
        Node::Element(el)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tag {
    // layout
    Table,
    TBody,
    Tr,
    Td,
    Div,
    Center,

    // text
    P,
    Span,
    Strong,
    Em,
    A,
    Br,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,

    // lists
    Ul,
    Ol,
    Li,

    // media
    Img,
}

impl Tag {
    pub fn name(self) -> &'static str {
        match self {
            Tag::Table => "table",
            Tag::TBody => "tbody",
            Tag::Tr => "tr",
            Tag::Td => "td",
            Tag::Div => "div",
            Tag::Center => "center",
            Tag::P => "p",
            Tag::Span => "span",
            Tag::Strong => "strong",
            Tag::Em => "em",
            Tag::A => "a",
            Tag::Br => "br",
            Tag::H1 => "h1",
            Tag::H2 => "h2",
            Tag::H3 => "h3",
            Tag::H4 => "h4",
            Tag::H5 => "h5",
            Tag::H6 => "h6",
            Tag::Ul => "ul",
            Tag::Ol => "ol",
            Tag::Li => "li",
            Tag::Img => "img",
        }
    }

    pub fn is_void(self) -> bool {
        matches!(self, Tag::Br | Tag::Img)
    }
}

/// The attributes whose value is a URL.
///
/// Deliberately disjoint from [`AttrName`]: these are reachable only via
/// [`Element::url_attr`], which demands a parsed [`Url`], so an unvalidated
/// string can never land in `href` or `src`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UrlAttr {
    Href,
    Src,
}

impl UrlAttr {
    pub fn name(self) -> &'static str {
        match self {
            UrlAttr::Href => "href",
            UrlAttr::Src => "src",
        }
    }
}

/// Attributes that carry a plain value. See [`UrlAttr`] for `href` and `src`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AttrName {
    Alt,
    Title,
    Target,
    Width,
    Height,
    Align,
    Valign,
    Bgcolor,
    Border,
    Cellpadding,
    Cellspacing,
    Colspan,
    Rowspan,
    Id,
    Role,
    Dir,
    Lang,
}

impl AttrName {
    pub fn name(self) -> &'static str {
        match self {
            AttrName::Alt => "alt",
            AttrName::Title => "title",
            AttrName::Target => "target",
            AttrName::Width => "width",
            AttrName::Height => "height",
            AttrName::Align => "align",
            AttrName::Valign => "valign",
            AttrName::Bgcolor => "bgcolor",
            AttrName::Border => "border",
            AttrName::Cellpadding => "cellpadding",
            AttrName::Cellspacing => "cellspacing",
            AttrName::Colspan => "colspan",
            AttrName::Rowspan => "rowspan",
            AttrName::Id => "id",
            AttrName::Role => "role",
            AttrName::Dir => "dir",
            AttrName::Lang => "lang",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AttrValue {
    Text(String),
    Int(u32),
    Url(Url),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Url(String);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UrlError {
    Empty,
    DisallowedScheme,
}

impl Url {
    /// Parses an absolute `http`, `https`, `mailto` or `tel` URL.
    ///
    /// Relative URLs are rejeceted, since an email has no base to resolve them against.
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StyleValue(String);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StyleMap {
    decls: Vec<(Property, StyleValue)>,
}

impl StyleMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, prop: Property, value: StyleValue) {
        match self.decls.iter_mut().find(|(p, _)| *p == prop) {
            Some((_, slot)) => *slot = value,
            None => self.decls.push((prop, value)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.decls.is_empty()
    }

    pub fn get(&self, prop: &Property) -> Option<&StyleValue> {
        self.decls.iter().find(|(p, _)| p == prop).map(|(_, v)| v)
    }

    pub fn declarations(&self) -> impl Iterator<Item = (&Property, &StyleValue)> {
        self.decls.iter().map(|(p, v)| (p, v))
    }

    pub fn fill_from(&mut self, other: &StyleMap) {
        for (prop, value) in &other.decls {
            if self.get(prop).is_none() {
                self.decls.push((prop.clone(), value.clone()));
            }
        }
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
