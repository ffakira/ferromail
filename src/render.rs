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
            out.push_str("<!--[if ");
            out.push_str(cond);
            out.push_str("]>");
            for child in children {
                node_to(child, out);
            }
            out.push_str("<![endif]-->");
        }
    }
}

fn element_to(el: &Element, out: &mut String) {
    let tag = el.tag();

    out.push('<');
    out.push_str(tag.name());

    for (name, value) in el.attrs() {
        out.push(' ');
        out.push_str(name.name());
        out.push_str("=\"");
        attr_value_to(value, out);
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
            '"' => out.push_str("&quote;"),
            '<' => out.push_str("&lt;"),
            _ => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::Tag;

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
}
