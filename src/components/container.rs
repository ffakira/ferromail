//! Centered, width-limited wrapper for email body content.

use super::{prop, styled};
use crate::markup::{AttrName, AttrValue, Color, Element, Node, Tag};

/// A centered column that fills a narrow viewport and stops growing on a wide one.
///
/// Two nested tables. The outer one spans the full width and centers the inner
/// one; the inner one carries both a `width` attribute and a `max-width`
/// declaration, which is what makes it fluid without a conditional comment.
///
/// Outlook's Word engine ignores `max-width` and honours the presentational
/// `width` attribute, so it lays out at the fixed width. Every other client
/// applies the declaration, which wins over the attribute, and the column
/// shrinks to fit. That is why there is no `[if mso]` ghost table here: the
/// two rules do not overlap, so nothing needs to be hidden from anyone.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Container {
    max_width_px: u32,
    padding_x_px: u32,
    padding_y_px: u32,
    background: Option<Color>,
    inner_background: Option<Color>,
    children: Vec<Node>,
}

impl Container {
    /// A 600px column padded 24px vertically and 16px horizontally.
    ///
    /// 600 is the width almost every email is designed to, and the one that
    /// fits Outlook's default reading pane without a horizontal scrollbar.
    pub fn new() -> Self {
        Self {
            max_width_px: 600,
            padding_x_px: 16,
            padding_y_px: 24,
            background: None,
            inner_background: None,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn max_width(mut self, px: u32) -> Self {
        self.max_width_px = px;
        self
    }

    /// Horizontal padding on the content cell.
    ///
    /// Inside the column, so it eats into the column rather than adding to it.
    /// Outside it would add to the total: Outlook lays the column out at the
    /// fixed `max_width`, so a 600px column with 16px either side would need
    /// 632px of reading pane and scroll horizontally in the one client this
    /// component is shaped around.
    ///
    /// On a narrow viewport the column has already shrunk to the full width,
    /// so the same padding is what keeps text off the screen edge.
    #[must_use]
    pub fn padding_x(mut self, px: u32) -> Self {
        self.padding_x_px = px;
        self
    }

    /// Vertical padding on the content cell.
    ///
    /// Padding rather than margin, because Outlook's Word engine ignores
    /// margin on a table cell. This is the only vertical space the column
    /// itself provides; spacing between siblings inside it is their own.
    #[must_use]
    pub fn padding_y(mut self, px: u32) -> Self {
        self.padding_y_px = px;
        self
    }

    /// Background behind the full width, outside the column.
    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Background behind the column itself.
    #[must_use]
    pub fn inner_background(mut self, color: Color) -> Self {
        self.inner_background = Some(color);
        self
    }

    #[must_use]
    pub fn child(mut self, node: Node) -> Self {
        self.children.push(node);
        self
    }

    #[must_use]
    pub fn children(mut self, nodes: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(nodes);
        self
    }

    /// Builds the wrapper.
    ///
    /// Infallible: colours were validated when they were set and every other
    /// value is generated from a `u32`.
    pub fn build(self) -> Node {
        let max_width = format!("{}px", self.max_width_px);
        let padding = format!("{} {}", px(self.padding_y_px), px(self.padding_x_px));

        let mut content = self
            .children
            .into_iter()
            .fold(Element::new(Tag::Td), Element::child);

        if self.padding_x_px > 0 || self.padding_y_px > 0 {
            content = styled(content, &[("padding", &padding)]);
        }

        let mut column = styled(
            Element::new(Tag::Table)
                .attr(AttrName::Role, AttrValue::Text("presentation".into()))
                .attr(AttrName::Border, AttrValue::Int(0))
                .attr(AttrName::Cellpadding, AttrValue::Int(0))
                .attr(AttrName::Cellspacing, AttrValue::Int(0))
                // Outlook reads this and ignores max-width below.
                .attr(AttrName::Width, AttrValue::Int(self.max_width_px))
                .attr(AttrName::Align, AttrValue::Text("center".into())),
            &[("width", "100%"), ("max-width", &max_width)],
        );

        if let Some(color) = &self.inner_background {
            column = column
                .attr(
                    AttrName::Bgcolor,
                    AttrValue::Text(color.as_str().to_owned()),
                )
                .style(prop("background-color"), color.style_value());
        }

        let column = column.child(Node::Element(
            Element::new(Tag::Tr).child(Node::Element(content)),
        ));

        let cell = Element::new(Tag::Td)
            .attr(AttrName::Align, AttrValue::Text("center".into()))
            .child(Node::Element(column));

        let mut outer = styled(
            Element::new(Tag::Table)
                .attr(AttrName::Role, AttrValue::Text("presentation".into()))
                .attr(AttrName::Border, AttrValue::Int(0))
                .attr(AttrName::Cellpadding, AttrValue::Int(0))
                .attr(AttrName::Cellspacing, AttrValue::Int(0))
                .attr(AttrName::Width, AttrValue::Text("100%".into())),
            &[("width", "100%")],
        );

        if let Some(color) = &self.background {
            outer = outer
                .attr(
                    AttrName::Bgcolor,
                    AttrValue::Text(color.as_str().to_owned()),
                )
                .style(prop("background-color"), color.style_value());
        }

        Node::Element(outer.child(Node::Element(
            Element::new(Tag::Tr).child(Node::Element(cell)),
        )))
    }
}

/// A zero length needs no unit, and `0px 16px` reads worse than `0 16px`.
fn px(n: u32) -> String {
    if n == 0 {
        "0".to_owned()
    } else {
        format!("{n}px")
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::render;

    fn color(hex: &str) -> Color {
        Color::hex(hex).expect("valid")
    }

    #[test]
    fn defaults_to_a_600px_column() {
        let html = render(&[Container::new().build()]);

        assert_eq!(
            html,
            concat!(
                r#"<table role="presentation" border="0" cellpadding="0" cellspacing="0""#,
                r#" width="100%" style="width:100%">"#,
                r#"<tr><td align="center">"#,
                r#"<table role="presentation" border="0" cellpadding="0" cellspacing="0""#,
                r#" width="600" align="center" style="width:100%;max-width:600px">"#,
                r#"<tr><td style="padding:24px 16px"></td></tr></table></td></tr></table>"#,
            )
        );
    }

    /// The pair that makes the column fluid without a ghost table: Outlook
    /// reads the attribute, everyone else applies the declaration.
    #[test]
    fn the_column_carries_both_a_width_attribute_and_max_width() {
        let html = render(&[Container::new().max_width(480).build()]);

        assert!(html.contains(r#"width="480""#), "{html}");
        assert!(html.contains("max-width:480px"), "{html}");
    }

    #[test]
    fn children_land_in_the_inner_cell() {
        let html = render(&[Container::new()
            .child(Node::Element(Element::new(Tag::P).text("hi")))
            .build()]);

        assert!(
            html.contains(r#"<tr><td style="padding:24px 16px"><p>hi</p></td></tr>"#),
            "{html}"
        );
    }

    #[test]
    fn backgrounds_are_stated_twice_and_on_different_tables() {
        let html = render(&[Container::new()
            .background(color("#f4f4f5"))
            .inner_background(color("#FFFFFF"))
            .build()]);

        assert!(html.contains(r##"bgcolor="#f4f4f5""##), "{html}");
        assert!(html.contains("background-color:#f4f4f5"), "{html}");
        assert!(html.contains(r##"bgcolor="#ffffff""##), "{html}");
        assert!(html.contains("background-color:#ffffff"), "{html}");

        // Outer before inner, so the page colour cannot paint over the column.
        let outer = html.find("#f4f4f5").expect("outer background");
        let inner = html.find("#ffffff").expect("inner background");
        assert!(outer < inner, "{html}");
    }

    /// Inside the column, not outside it. Outside, the padding would add to
    /// the fixed width Outlook lays the column out at, so a 600px column with
    /// horizontal padding would need more than 600px of reading pane.
    #[test]
    fn the_padding_is_inside_the_column() {
        let html = render(&[Container::new().padding_x(24).padding_y(0).build()]);

        assert!(html.contains(r#"<td align="center"><table"#), "{html}");
        assert!(html.contains(r#"<td style="padding:0 24px">"#), "{html}");

        let column = html.find(r#"width="600""#).expect("column");
        let padding = html.find("padding:0 24px").expect("padding");
        assert!(padding > column, "padding is outside the column: {html}");
    }

    #[test]
    fn a_zero_padding_emits_no_declaration() {
        let html = render(&[Container::new().padding_x(0).padding_y(0).build()]);

        // Not `contains("padding")`: cellpadding="0" would match that.
        assert!(!html.contains("padding:"), "{html}");
        assert!(html.contains("<tr><td></td></tr>"), "{html}");
    }

    #[test]
    fn one_axis_alone_still_emits_the_shorthand() {
        let only_y = render(&[Container::new().padding_x(0).padding_y(32).build()]);
        assert!(only_y.contains(r#"style="padding:32px 0""#), "{only_y}");

        let only_x = render(&[Container::new().padding_x(32).padding_y(0).build()]);
        assert!(only_x.contains(r#"style="padding:0 32px""#), "{only_x}");
    }
}
