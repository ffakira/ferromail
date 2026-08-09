//! The element names this crate can emit.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tag {
    // document
    Html,
    Head,
    Body,
    Meta,
    Title,
    Style,

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

    // vml, Outlook only, emitted inside an mso conditional
    VRoundRect,
    WAnchorLock,
}

impl Tag {
    pub fn name(self) -> &'static str {
        match self {
            Tag::Html => "html",
            Tag::Head => "head",
            Tag::Body => "body",
            Tag::Meta => "meta",
            Tag::Title => "title",
            Tag::Style => "style",
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
            Tag::VRoundRect => "v:roundrect",
            Tag::WAnchorLock => "w:anchorlock",
        }
    }

    /// Whether the tag renders as `<x />` with no closing tag.
    ///
    /// Children of a void element are dropped when rendered. See
    /// [`Element::child`](super::Element::child).
    pub fn is_void(self) -> bool {
        matches!(self, Tag::Br | Tag::Img | Tag::Meta | Tag::WAnchorLock)
    }
}
