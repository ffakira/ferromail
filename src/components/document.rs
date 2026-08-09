//! The `<html>` wrapper an email needs before VML will render.

use super::styled;
use crate::markup::{AttrName, AttrValue, Element, Node, Stylesheet, Tag};

/// VML namespaces, required on `<html>` for `v:` and `w:` elements to render
/// in Outlook's Word engine. Without them the whole VML branch is inert.
const VML_NS: &str = "urn:schemas-microsoft-com:vml";
const WORD_NS: &str = "urn:schemas-microsoft-com:office:word";

/// A complete email document.
///
/// Declares the VML namespaces unconditionally, so any [`Button`] placed in
/// the body renders in Outlook as well as everywhere else.
///
/// [`Button`]: super::Button
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Document {
    title: String,
    language: String,
    stylesheet: Stylesheet,
    children: Vec<Node>,
}

impl Document {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            language: "en".to_owned(),
            stylesheet: Stylesheet::new(),
            children: Vec::new(),
        }
    }

    /// The `lang` attribute on `<html>`, defaulting to `en`.
    ///
    /// A screen reader picks its pronunciation from this, so the default is a
    /// guess that is wrong for most of the world. Set it.
    #[must_use]
    pub fn language(mut self, tag: impl Into<String>) -> Self {
        self.language = tag.into();
        self
    }

    /// Sets the `<style>` block in the head.
    ///
    /// Use it for media queries, and treat it as an enhancement: Gmail's app
    /// strips `<style>` for non-Gmail accounts and Outlook's Word engine
    /// ignores media queries, so the inline layout has to stand on its own.
    #[must_use]
    pub fn stylesheet(mut self, sheet: Stylesheet) -> Self {
        self.stylesheet = sheet;
        self
    }

    #[must_use]
    pub fn child(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }

    /// Appends several nodes, which is what [`Button::build`] returns.
    ///
    /// [`Button::build`]: super::Button::build
    #[must_use]
    pub fn children(mut self, nodes: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(nodes);
        self
    }

    /// Builds the doctype and the `<html>` element.
    ///
    /// Two nodes, because a doctype has to precede `<html>` and is not an
    /// element. Infallible: every value is crate-owned or escaped at render
    /// time.
    pub fn build(self) -> Vec<Node> {
        let head = Element::new(Tag::Head)
            .child(Node::Element(
                Element::new(Tag::Meta).attr(AttrName::Charset, AttrValue::Text("utf-8".into())),
            ))
            .child(Node::Element(
                Element::new(Tag::Meta)
                    .attr(AttrName::Name, AttrValue::Text("viewport".into()))
                    .attr(
                        AttrName::Content,
                        AttrValue::Text("width=device-width, initial-scale=1".into()),
                    ),
            ))
            .child(Node::Element(
                Element::new(Tag::Title).text(self.title.clone()),
            ))
            .child(Node::Style(self.stylesheet.clone()));

        // Clients that keep <body> apply a default margin. Those that strip
        // it ignore this, which is why Container exists as well.
        let body = self.children.into_iter().fold(
            styled(
                Element::new(Tag::Body),
                &[("margin", "0"), ("padding", "0"), ("width", "100%")],
            ),
            Element::child,
        );

        vec![
            Node::Doctype,
            Node::Element(
                Element::new(Tag::Html)
                    .attr(AttrName::Lang, AttrValue::Text(self.language.clone()))
                    .attr(AttrName::XmlnsV, AttrValue::Text(VML_NS.into()))
                    .attr(AttrName::XmlnsW, AttrValue::Text(WORD_NS.into()))
                    .child(Node::Element(head))
                    .child(Node::Element(body)),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Button;
    use crate::markup::Url;
    use crate::render::render;

    #[test]
    fn declares_the_vml_namespaces() {
        let html = render(&Document::new("Hi").build());

        assert!(
            html.contains(r#"xmlns:v="urn:schemas-microsoft-com:vml""#),
            "{html}"
        );
        assert!(
            html.contains(r#"xmlns:w="urn:schemas-microsoft-com:office:word""#),
            "{html}"
        );
    }

    #[test]
    fn starts_with_a_doctype_before_the_html_element() {
        let html = render(&Document::new("Hi").build());

        assert!(html.starts_with("<!DOCTYPE html PUBLIC"), "{html}");
        let doctype = html.find("<!DOCTYPE").expect("doctype");
        let root = html.find("<html").expect("html");
        assert!(doctype < root, "{html}");
    }

    #[test]
    fn language_defaults_to_en_and_can_be_set() {
        let default = render(&Document::new("Hi").build());
        assert!(default.contains(r#"<html lang="en""#), "{default}");

        let set = render(&Document::new("Oi").language("pt-BR").build());
        assert!(set.contains(r#"<html lang="pt-BR""#), "{set}");
    }

    /// Clients that keep <body> apply a default margin. The ones that strip it
    /// are why Container exists as well, so this is a complement, not a fix.
    #[test]
    fn body_carries_a_reset() {
        let html = render(&Document::new("Hi").build());
        assert!(
            html.contains(r#"<body style="margin:0;padding:0;width:100%">"#),
            "{html}"
        );
    }

    #[test]
    fn title_is_escaped() {
        let html = render(&Document::new("a < b & c").build());
        assert!(html.contains("<title>a &lt; b &amp; c</title>"), "{html}");
    }

    #[test]
    fn a_button_in_the_body_has_its_namespaces() {
        let href = Url::parse("https://example.com").expect("valid");
        let html = render(
            &Document::new("Confirm")
                .children(Button::new(href, "Confirm").build())
                .build(),
        );

        // The namespace declaration must precede the element that needs it.
        let ns = html.find("xmlns:v=").expect("namespace present");
        let vml = html.find("<v:roundrect").expect("vml present");
        assert!(ns < vml, "namespace declared after the VML element: {html}");
    }
}
