use crate::markup::{AttrValue, Element, Node};

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
