pub struct RawHtml(String);

impl RawHtml {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub enum Node {
    Element(Element),
    Text(String),
    Raw(RawHtml),
    Conditional { cond: String, children: Vec<Node> },
}

pub struct Element {
    tag: Tag,
    attrs: Vec<(AttrName, AttrValue)>,
    class: Vec<ClassName>,
    styles: StyleMap,
    children: Vec<Node>,
}

impl Element {
    pub fn new(tag: Tag) -> Self {
        Self {
            tag,
            attrs: Vec::new(),
            class: Vec::new(),
            styles: StyleMap::new(),
            children: Vec::new(),
        }
    }

    pub fn attr(mut self, name: AttrName, value: AttrValue) -> Self {
        self.attrs.retain(|(n, _)| *n != name);
        self.attrs.push((name, value));
        self
    }

    pub fn child(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }

    pub fn text(self, text: impl Into<String>) -> Self {
        self.child(Node::Text(text.into()))
    }

    pub fn tag(&self) -> Tag {
        self.tag
    }

    pub fn attrs(&self) -> &[(AttrName, AttrValue)] {
        &self.attrs
    }

    pub fn class(&self) -> &[ClassName] {
        &self.class
    }

    pub fn styles(&self) -> &StyleMap {
        &self.styles
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AttrName {
    Href,
    Src,
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
            AttrName::Href => "href",
            AttrName::Src => "src",
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

    pub fn is_url(self) -> bool {
        matches!(self, AttrName::Href | AttrName::Src)
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

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StyleMap {
    decls: Vec<(Property, String)>,
}

impl StyleMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, prop: Property, value: impl Into<String>) {
        let value = value.into();
        match self.decls.iter_mut().find(|(p, _)| *p == prop) {
            Some((_, slot)) => *slot = value,
            None => self.decls.push((prop, value)),
        }
    }

    pub fn get(&self, prop: &Property) -> Option<&str> {
        self.decls
            .iter()
            .find(|(p, _)| p == prop)
            .map(|(_, v)| v.as_str())
    }

    pub fn declarations(&self) -> impl Iterator<Item = (&Property, &str)> {
        self.decls.iter().map(|(p, v)| (p, v.as_str()))
    }

    pub fn fill_ftrom(&mut self, other: &StyleMap) {
        for (prop, value) in &other.decls {
            if self.get(prop).is_none() {
                self.decls.push((prop.clone(), value.clone()));
            }
        }
    }
}
