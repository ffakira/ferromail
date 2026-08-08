//! Property tests over the public API.
//!
//! The invariant: a tree built without `RawHTML` can never emit a tag or an
//! attribute that was not push there deliberately, no matter what strings are
//! fed into text, attribute values, classes or styles.

use ferromail::markup::{
    AttrName, AttrValue, ClassName, Element, Node, Property, StyleValue, Tag, Url, UrlAttr,
};
use ferromail::render::render;
use proptest::prelude::*;

const TAGS: &[Tag] = &[
    // VML: namespaced, emitted inside conditionals, and the newest markup
    // path — exactly where the escaping assumptions are least exercised.
    Tag::VRoundRect,
    Tag::WAnchorLock,
    Tag::Div,
    Tag::Span,
    Tag::P,
    Tag::A,
    Tag::Img,
    Tag::Table,
    Tag::Tr,
    Tag::Td,
    Tag::Br,
];

const ATTR_NAMES: &[AttrName] = &[
    AttrName::Alt,
    AttrName::Title,
    AttrName::Width,
    AttrName::Align,
    AttrName::Bgcolor,
    AttrName::ArcSize,
    AttrName::FillColor,
    AttrName::StrokeColor,
    AttrName::XmlnsV,
    AttrName::Charset,
];

const URL_ATTRS: &[UrlAttr] = &[UrlAttr::Href, UrlAttr::Src];

/// Only parsed URLs can reach href/src, so generate them the same way a
/// caller must: hostile string in, `Url::parse` as the only gate.
fn url() -> impl Strategy<Value = Url> {
    (
        prop::sample::select(&["https", "http", "mailto", "tel"][..]),
        "[a-zA-Z0-9./?=&#\"'<> -]{0,24}",
    )
        .prop_filter_map("valid url", |(scheme, rest)| {
            Url::parse(&format!("{scheme}:{rest}")).ok()
        })
}

/// Nasty strings that should never survive into the output as markup.
fn hostile_text() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::from("<script>alert(1)</script>")),
        Just(String::from("\" onerror=\"alert(1)")),
        Just(String::from("' onload='x")),
        Just(String::from("</td><script>x</script>")),
        Just(String::from("&lt;script&gt;")),
        Just(String::from("&#60;script&#62;")),
        Just(String::from("]]><!--")),
        Just(String::from("--><script>x</script><!--")),
        ".*",
    ]
}

fn attr_value() -> impl Strategy<Value = AttrValue> {
    prop_oneof![
        hostile_text().prop_map(AttrValue::Text),
        any::<u32>().prop_map(AttrValue::Int),
        (
            prop::sample::select(&["https", "http", "mailto", "tel"][..]),
            "[a-zA-Z0-9./?=&#\"'<> -]{0,24}",
        )
            .prop_filter_map("valid url", |(scheme, rest)| {
                Url::parse(&format!("{scheme}:{rest}"))
                    .ok()
                    .map(AttrValue::Url)
            }),
    ]
}

fn element() -> impl Strategy<Value = Element> {
    (
        prop::sample::select(TAGS),
        prop::collection::vec((prop::sample::select(ATTR_NAMES), attr_value()), 0..4),
        prop::collection::vec((prop::sample::select(URL_ATTRS), url()), 0..3),
        prop::collection::vec(".*", 0..3),
        prop::collection::vec((".*", ".*"), 0..3),
    )
        .prop_map(|(tag, attrs, urls, classes, styles)| {
            let mut el = Element::new(tag);
            for (name, value) in attrs {
                el = el.attr(name, value);
            }

            for (name, u) in urls {
                el = el.url_attr(name, u);
            }

            for c in classes {
                if let Some(c) = ClassName::new(&c) {
                    el = el.class(c);
                }
            }

            for (p, v) in styles {
                if let (Some(p), Ok(v)) = (Property::new(&p), StyleValue::parse(&v)) {
                    el = el.style(p, v);
                }
            }
            el
        })
}

fn node() -> impl Strategy<Value = Node> {
    let leaf = prop_oneof![
        hostile_text().prop_map(Node::Text),
        element().prop_map(Node::Element),
    ];

    leaf.prop_recursive(3, 24, 3, |inner| {
        (
            element().prop_filter("container", |el| !el.tag().is_void()),
            prop::collection::vec(inner, 0..3),
        )
            .prop_map(|(el, kids)| {
                let mut el = el;
                for kid in kids {
                    el = el.child(kid);
                }
                Node::Element(el)
            })
            .boxed()
    })
}

/// Collects every `<name` and `</name` token the output actually contains
fn tags_in(html: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == '<' {
            let mut j = i + 1;
            if j < bytes.len() && bytes[j] == '/' {
                j += 1;
            }

            // ':' belongs to the name: VML tags are namespaced (`v:roundrect`),
            // and stopping at the colon would both under-report them and let a
            // smuggled `<v:...>` past this check.
            let mut name = String::new();
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == '-' || bytes[j] == ':')
            {
                name.push(bytes[j]);
                j += 1;
            }
            if !name.is_empty() {
                found.push(name.to_ascii_lowercase());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    found
}

/// Every tag the tree declares, counted the way the renderer emits them.
fn expected_tags(node: &Node, out: &mut Vec<String>) {
    match node {
        Node::Element(el) => {
            let name = el.tag().name().to_owned();
            out.push(name.clone());

            if !el.tag().is_void() {
                out.push(name);
            }

            for child in el.children() {
                expected_tags(child, out);
            }
        }
        Node::Conditional { children, .. } => {
            for child in children {
                expected_tags(child, out);
            }
        }
        Node::Text(_) | Node::Raw(_) | Node::Style(_) => {}
    }
}

/// Counts `"` characters that appear inside a tag. Every one should be an
/// attribute delimiter; a value that escaped is quoting shows up as a surplus.
fn quotes_inside_tags(html: &str) -> usize {
    let mut count = 0;
    let mut in_tag = false;
    let mut in_quote = false;

    for c in html.chars() {
        match c {
            '<' if !in_tag => in_tag = true,
            '>' if in_tag && !in_quote => in_tag = false,
            '"' if in_tag => {
                count += 1;
                in_quote = !in_quote;
            }
            _ => {}
        }
    }
    count
}

fn expected_quotes(node: &Node) -> usize {
    match node {
        Node::Element(el) => {
            let mut n = el.attrs().len() + el.urls().len();
            if !el.classes().is_empty() {
                n += 1;
            }

            if !el.styles().is_empty() {
                n += 1;
            }

            n * 2 + el.children().iter().map(expected_quotes).sum::<usize>()
        }
        Node::Conditional { children, .. } => children.iter().map(expected_quotes).sum(),
        Node::Text(_) | Node::Raw(_) | Node::Style(_) => 0,
    }
}

proptest! {
    #[test]
    fn attribute_values_stay_inside_their_quotes(node in node()) {
        let html = render(std::slice::from_ref(&node));
        prop_assert_eq!(
            quotes_inside_tags(&html),
            expected_quotes(&node),
            "output was: {}", html
        );
    }

    #[test]
    fn no_smuggled_tags(node in node()) {
        let html = render(std::slice::from_ref(&node));

        let mut expected = Vec::new();
        expected_tags(&node, &mut expected);
        expected.sort();

        let mut actual = tags_in(&html);
        actual.sort();

        prop_assert_eq!(actual, expected, "output was: {}", html);
    }

    #[test]
    fn text_never_introduces_markup(texts in prop::collection::vec(hostile_text(), 1..8)) {
        let nodes: Vec<Node> = texts.into_iter().map(Node::Text).collect();
        let html = render(&nodes);

        prop_assert!(!html.contains('<'), "text leaked a tag: {}", html);
        prop_assert!(!html.contains('>'), "text leaked a tag: {}", html);
    }
}
