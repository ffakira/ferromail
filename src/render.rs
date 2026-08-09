//! Turning a [`Node`] tree into HTML.
//!
//! This is where escaping happens, and it is the only place that writes to the
//! output. Everything it emits verbatim is safe because of the type it came
//! from, not because of a check performed here. See [`render`].

use crate::markup::{AttrValue, Element, Node};

/// Serialises a node tree to HTML.
///
/// The output is a single line with no whitespace between elements. Gmail
/// clips a message once it passes 102KB and hides everything after the cut,
/// so bytes spent on indentation are bytes not spent on content.
///
/// ```
/// use ferromail::markup::{Element, Node, Tag};
/// use ferromail::render::render;
///
/// let el = Element::new(Tag::P).text("5 > 3 & rising");
/// assert_eq!(render(&[Node::Element(el)]), "<p>5 &gt; 3 &amp; rising</p>");
/// ```
///
/// # Escaping
///
/// Text and attribute values are escaped here, which is what stops a string
/// that came from outside the program becoming markup.
///
/// Four things are written verbatim, and each is safe by construction rather
/// than by a check performed at this point:
///
/// - [`RawHtml`](crate::markup::RawHtml) is the deliberate escape hatch. Its
///   only constructor is
///   [`RawHtml::trusted`](crate::markup::RawHtml::trusted), named so the claim
///   the caller is making shows up in review.
/// - [`Stylesheet`](crate::markup::Stylesheet) is typed down to the selector,
///   so `</style>` cannot occur inside a `<style>` block.
/// - [`ClassName`](crate::markup::ClassName) rejects quotes and angle
///   brackets, so a class cannot close the `class` attribute.
/// - [`Property`](crate::markup::Property) and
///   [`StyleValue`](crate::markup::StyleValue) reject the characters that
///   could end a declaration or the `style` attribute.
///
/// Conditional comments are emitted from [`Condition`](crate::markup::Condition)
/// rather than a string, so `!mso` gets the downlevel-revealed form and a
/// condition cannot close the comment early.
///
/// # Void elements
///
/// A void tag renders as `<x />` and its children are dropped. See
/// [`Element::child`](crate::markup::Element::child).
pub fn render(nodes: &[Node]) -> String {
    let mut out = String::new();
    for node in nodes {
        node_to(node, &mut out);
    }
    out
}

fn node_to(node: &Node, out: &mut String) {
    match node {
        Node::Text(text) => escape_text(text, out),
        Node::Raw(raw) => out.push_str(raw.as_str()),
        Node::Element(el) => element_to(el, out),
        // XHTML 1.0 Transitional rather than the HTML5 one, because the
        // renderer closes void elements as `<img />`, which this doctype
        // requires and HTML5 merely tolerates.
        Node::Doctype => out.push_str(
            "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \
             \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">",
        ),
        // Not escaped, and it does not need to be: every part of a Stylesheet
        // is a ClassName, Property or StyleValue, all of which reject `<` and
        // `>`, so `</style>` cannot occur. An empty sheet emits nothing rather
        // than a bare <style></style>.
        Node::Style(sheet) => {
            if !sheet.is_empty() {
                out.push_str("<style>");
                out.push_str(&sheet.to_css());
                out.push_str("</style>");
            }
        }
        Node::Conditional { cond, children } => {
            if cond.is_revealed() {
                out.push_str("<!--[if ");
                out.push_str(&cond.expr());
                out.push_str("]><!-->");
            } else {
                out.push_str("<!--[if ");
                out.push_str(&cond.expr());
                out.push_str("]>");
            }

            for child in children {
                node_to(child, out);
            }

            if cond.is_revealed() {
                out.push_str("<!--<![endif]-->");
            } else {
                out.push_str("<![endif]-->");
            }
        }
    }
}

fn element_to(el: &Element, out: &mut String) {
    let tag = el.tag();

    out.push('<');
    out.push_str(tag.name());

    for (name, url) in el.urls() {
        out.push(' ');
        out.push_str(name.name());
        out.push_str("=\"");
        escape_attr(url.as_str(), out);
        out.push('"');
    }

    for (name, value) in el.attrs() {
        out.push(' ');
        out.push_str(name.name());
        out.push_str("=\"");
        attr_value_to(value, out);
        out.push('"');
    }

    // No escaping below: ClassName, Property and StyleValue all reject the
    // characters that could close the attribute, so they are safe by
    // construction.
    if !el.classes().is_empty() {
        out.push_str(" class=\"");
        for (i, class) in el.classes().iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(class.as_str());
        }
        out.push('"');
    }

    if !el.styles().is_empty() {
        out.push_str(" style=\"");
        for (i, (prop, value)) in el.styles().declarations().enumerate() {
            if i > 0 {
                out.push(';');
            }
            out.push_str(prop.as_str());
            out.push(':');
            out.push_str(value.as_str());
        }
        out.push('"');
    }

    // A void tag has no closing tag, so any children it was given are
    // dropped here. See `Element::child`. This is stated behaviour, not an
    // oversight, and `void_tag_children_are_dropped` pins it.
    if tag.is_void() {
        out.push_str(" />");
        return;
    }

    out.push('>');
    for child in el.children() {
        node_to(child, out);
    }

    out.push_str("</");
    out.push_str(tag.name());
    out.push('>');
}

fn attr_value_to(value: &AttrValue, out: &mut String) {
    match value {
        AttrValue::Int(n) => out.push_str(&n.to_string()),
        AttrValue::Text(text) => escape_attr(text, out),
        AttrValue::Url(url) => escape_attr(url.as_str(), out),
    }
}

fn escape_text(raw: &str, out: &mut String) {
    for c in raw.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

fn escape_attr(raw: &str, out: &mut String) {
    for c in raw.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            _ => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::{
        AttrName, ClassName, Condition, Property, RawHtml, StyleValue, Tag, Url, UrlAttr,
    };

    #[test]
    fn escape_text() {
        let nodes = vec![Node::Text("a < b & c".into())];
        assert_eq!(render(&nodes), "a &lt; b &amp; c");
    }

    #[test]
    fn void_tags_have_no_closing_tag() {
        let nodes = vec![Node::Element(Element::new(Tag::Img))];
        assert_eq!(render(&nodes), "<img />");
    }

    #[test]
    fn not_mso_is_downlevel_revealed() {
        let nodes = vec![Node::Conditional {
            cond: Condition::NotMso,
            children: vec![Node::Text("everyone else".into())],
        }];
        assert_eq!(
            render(&nodes),
            "<!--[if !mso]><!-->everyone else<!--<![endif]-->"
        );
    }

    #[test]
    fn quotes_in_attr_cannot_close_it() {
        let nodes = vec![Node::Element(
            Element::new(Tag::Img).attr(AttrName::Alt, AttrValue::Text(r#"a " onerror=x"#.into())),
        )];
        assert_eq!(render(&nodes), r#"<img alt="a &quot; onerror=x" />"#);
    }

    #[test]
    fn url_attrs_come_from_parsed_urls() {
        let el = Element::new(Tag::A)
            .url_attr(
                UrlAttr::Href,
                Url::parse("https://x.com/a?b=1&c=2").expect("valid"),
            )
            .text("hi");

        assert_eq!(
            render(&[Node::Element(el)]),
            r#"<a href="https://x.com/a?b=1&amp;c=2">hi</a>"#
        );

        // The scheme allowlist is the only door to href, and it is shut.
        assert!(Url::parse("javascript:alert(1)").is_err());
    }

    /// Deliberate, documented behaviour: a void element has no closing tag,
    /// so anything `child` put on it is discarded here. Pinned so the choice
    /// cannot drift into an accident.
    #[test]
    fn void_tag_children_are_dropped() {
        let el = Element::new(Tag::Br).child(Node::Text("dropped".into()));
        assert_eq!(render(&[Node::Element(el)]), "<br />");

        let el = Element::new(Tag::Img).child(Node::Element(Element::new(Tag::Div)));
        assert_eq!(render(&[Node::Element(el)]), "<img />");

        // Attributes on a void element still render; only children are lost.
        let el = Element::new(Tag::Br)
            .attr(AttrName::Id, AttrValue::Text("x".into()))
            .child(Node::Text("dropped".into()));
        assert_eq!(render(&[Node::Element(el)]), r#"<br id="x" />"#);
    }

    #[test]
    fn raw_is_verbatim() {
        let nodes = vec![Node::Raw(RawHtml::trusted("<b>hi</b>"))];
        assert_eq!(render(&nodes), "<b>hi</b>");
    }

    #[test]
    fn renders_class_and_style() {
        let el = Element::new(Tag::Td)
            .class(ClassName::new("stack").expect("valid"))
            .style(
                Property::new("color").expect("valid"),
                StyleValue::parse("#333").expect("valid"),
            )
            .text("hi");

        assert_eq!(
            render(&[Node::Element(el)]),
            r#"<td class="stack" style="color:#333">hi</td>"#
        );
    }
}
