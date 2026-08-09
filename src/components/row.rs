//! Horizontal blocks, optionally split into columns.

use super::{Column, prop, styled};
use crate::markup::{AttrName, AttrValue, Color, Element, Node, Tag};

/// A full-width block, holding one or more [`Column`]s side by side.
///
/// Its own table rather than a bare `<tr>`, so a row composes anywhere a node
/// is accepted rather than only inside another table.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Row {
    columns: Vec<Column>,
    background: Option<Color>,
    stack: bool,
}

impl Row {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            background: None,
            stack: false,
        }
    }

    /// A row of one full-width column holding these nodes.
    #[must_use]
    pub fn single(nodes: impl IntoIterator<Item = Node>) -> Self {
        Self::new().column(Column::new().width(100).children(nodes))
    }

    #[must_use]
    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    #[must_use]
    pub fn columns(mut self, columns: impl IntoIterator<Item = Column>) -> Self {
        self.columns.extend(columns);
        self
    }

    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Marks the columns to stack on a narrow viewport.
    ///
    /// Does nothing on its own: [`stack_rules`](super::stack_rules) has to reach the document
    /// stylesheet for the class to mean anything.
    #[must_use]
    pub fn stack(mut self) -> Self {
        self.stack = true;
        self
    }

    /// Builds the row.
    ///
    /// Infallible: colours were validated when they were set and every other
    /// value is generated from a `u32`.
    pub fn build(self) -> Node {
        let stacking = self.stack;

        let tr = self
            .columns
            .into_iter()
            .map(|c| Node::Element(c.build(stacking)))
            .fold(Element::new(Tag::Tr), Element::child);

        let mut table = styled(
            Element::new(Tag::Table)
                .attr(AttrName::Role, AttrValue::Text("presentation".to_owned()))
                .attr(AttrName::Border, AttrValue::Int(0))
                .attr(AttrName::Cellpadding, AttrValue::Int(0))
                .attr(AttrName::Cellspacing, AttrValue::Int(0))
                .attr(AttrName::Width, AttrValue::Text("100%".to_owned())),
            &[("width", "100%"), ("border-collapse", "collapse")],
        );

        if let Some(color) = &self.background {
            table = table
                .attr(
                    AttrName::Bgcolor,
                    AttrValue::Text(color.as_str().to_owned()),
                )
                .style(prop("background-color"), color.style_value());
        }

        Node::Element(table.child(Node::Element(tr)))
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::column::STACK_CLASS;
    use crate::render::render;

    fn color(hex: &str) -> Color {
        Color::hex(hex).expect("valid")
    }

    fn text(s: &str) -> Node {
        Node::Element(Element::new(Tag::P).text(s))
    }

    #[test]
    fn two_columns_render_side_by_side() {
        let html = render(&[Row::new()
            .column(Column::new().width(50).child(text("left")))
            .column(Column::new().width(50).child(text("right")))
            .build()]);

        assert!(
            html.contains(r#"<tr><td valign="top" width="50%" style="width:50%"><p>left</p></td>"#),
            "{html}"
        );
        assert!(html.matches("<td ").count() == 2, "{html}");
    }

    #[test]
    fn single_is_one_full_width_column() {
        let html = render(&[Row::single([text("only")]).build()]);

        assert!(html.contains(r#"width="100%""#), "{html}");
        assert!(html.matches("<td ").count() == 1, "{html}");
    }

    #[test]
    fn stack_marks_every_column_and_is_off_by_default() {
        let plain = render(&[Row::new().column(Column::new()).build()]);
        assert!(!plain.contains(STACK_CLASS), "{plain}");

        let stacked = render(&[Row::new()
            .column(Column::new())
            .column(Column::new())
            .stack()
            .build()]);
        assert_eq!(stacked.matches(STACK_CLASS).count(), 2, "{stacked}");
    }

    #[test]
    fn backgrounds_are_stated_twice_on_row_and_column() {
        let html = render(&[Row::new()
            .background(color("#111111"))
            .column(Column::new().background(color("#222222")))
            .build()]);

        assert!(html.contains(r##"bgcolor="#111111""##), "{html}");
        assert!(html.contains("background-color:#111111"), "{html}");
        assert!(html.contains(r##"bgcolor="#222222""##), "{html}");
        assert!(html.contains("background-color:#222222"), "{html}");
    }
}
