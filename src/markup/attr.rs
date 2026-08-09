//! Attribute names and values.
//!
//! The split between [`UrlAttr`] and [`AttrName`] is what keeps an
//! unvalidated string out of `href` and `src`.

use super::Url;

/// The attributes whose value is a URL.
///
/// Deliberately disjoint from [`AttrName`]: these are reachable only via
/// [`Element::url_attr`], which demands a parsed [`Url`], so an unvalidated
/// string can never land in `href` or `src`.
///
/// [`Element::url_attr`]: super::Element::url_attr
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
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
#[non_exhaustive]
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

    // document head
    Charset,
    Content,
    Name,

    // vml
    ArcSize,
    FillColor,
    StrokeColor,
    XmlnsV,
    XmlnsW,
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
            AttrName::Charset => "charset",
            AttrName::Content => "content",
            AttrName::Name => "name",
            AttrName::ArcSize => "arcsize",
            AttrName::FillColor => "fillcolor",
            AttrName::StrokeColor => "strokecolor",
            AttrName::XmlnsV => "xmlns:v",
            AttrName::XmlnsW => "xmlns:w",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum AttrValue {
    Text(String),
    Int(u32),
    Url(Url),
}

// Conversions exist so `html!` can accept `align=("center")` and
// `border=(0)` without the caller naming the variant. Text is still escaped
// on render, so these are ergonomic only: they open no new path.

impl From<&str> for AttrValue {
    fn from(text: &str) -> Self {
        AttrValue::Text(text.to_owned())
    }
}

impl From<String> for AttrValue {
    fn from(text: String) -> Self {
        AttrValue::Text(text)
    }
}

impl From<u32> for AttrValue {
    fn from(n: u32) -> Self {
        AttrValue::Int(n)
    }
}

impl From<Url> for AttrValue {
    fn from(url: Url) -> Self {
        AttrValue::Url(url)
    }
}
