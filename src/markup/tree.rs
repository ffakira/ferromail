//! The node tree: elements, text, trusted raw markup and Outlook conditionals.

use std::borrow::Cow;

use super::{AttrName, AttrValue, ClassName, Property, StyleMap, StyleValue, Tag, Url, UrlAttr};

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

    /// Appends a child.
    ///
    /// If the tag is void ([`Tag::is_void`]) the child is **dropped at render
    /// time**, because a void element has no closing tag to hold it. The
    /// builder accepts it rather than refusing, so this is a silent no-op:
    /// `Element::new(Tag::Br).child(..)` renders as plain `<br />`.
    ///
    /// Making that unrepresentable needs the void-ness in the element's own
    /// type — a second type or a typestate parameter, which would leak
    /// through `Node`, the renderer and every component. That is not worth it
    /// for a mistake whose symptom is visibly missing content. Revisit if it
    /// ever bites in practice.
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
