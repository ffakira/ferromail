//! One cell in a [`Row`](super::Row), and the stacking it opts into.

use super::{prop, px, styled, value};
use crate::markup::{AttrName, AttrValue, ClassName, Color, Element, Node, Rule, Tag};

/// The class a stacking column carries.
///
/// Exposed because the rule that acts on it lives in the document stylesheet,
/// not here. See [`stack_rules`].
pub const STACK_CLASS: &str = "fm-stack";

/// The rules that make [`Row::stack`](super::Row::stack) do anything.
///
/// Add them to the document stylesheet:
///
/// ```
/// use ferromail::components::stack_rules;
/// use ferromail::markup::{MediaQuery, Stylesheet};
///
/// let sheet = Stylesheet::new().media(MediaQuery::MaxWidth(600), stack_rules());
/// assert!(sheet.to_css().contains(".fm-stack"));
/// ```
///
/// Stacking depends on a `<style>` block, so it does not happen in clients
/// that strip one, notably Gmail's app on a non-Gmail account. Those clients
/// keep the columns side by side, which is narrow but not broken. If that
/// matters more than the layout, use one column per row.
///
/// # Panics
///
/// Never in practice: every value is a crate literal, checked by tests.
#[must_use]
pub fn stack_rules() -> Vec<Rule> {
    let class = ClassName::new(STACK_CLASS).expect("literal");

    vec![
        Rule::class(class)
            .set(prop("display"), value("block"))
            .set(prop("width"), value("100%"))
            .set(prop("max-width"), value("100%"))
            .important(),
    ]
}

/// One cell in a [`Row`](super::Row).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Column {
    width_pct: Option<u32>,
    padding_x_px: u32,
    padding_y_px: u32,
    background: Option<Color>,
    valign: &'static str,
    children: Vec<Node>,
}

impl Column {
    pub fn new() -> Self {
        Self {
            width_pct: None,
            padding_x_px: 0,
            padding_y_px: 0,
            background: None,
            valign: "top",
            children: Vec::new(),
        }
    }

    /// Share of the row, as a percentage.
    ///
    /// Percentages rather than pixels so the row still divides correctly once
    /// the container has shrunk below its `max_width`.
    #[must_use]
    pub fn width(mut self, pct: u32) -> Self {
        self.width_pct = Some(pct);
        self
    }

    #[must_use]
    pub fn padding_x(mut self, px: u32) -> Self {
        self.padding_x_px = px;
        self
    }

    #[must_use]
    pub fn padding_y(mut self, px: u32) -> Self {
        self.padding_y_px = px;
        self
    }

    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Vertical alignment of the cell, `top` by default.
    ///
    /// Cells in a row are as tall as the tallest one, so without this a short
    /// column centres itself against a long neighbour.
    #[must_use]
    pub fn valign_middle(mut self) -> Self {
        self.valign = "middle";
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

    pub(crate) fn build(self, stacking: bool) -> Element {
        let mut cell = self
            .children
            .into_iter()
            .fold(Element::new(Tag::Td), Element::child)
            .attr(AttrName::Valign, AttrValue::Text(self.valign.to_owned()));

        if let Some(pct) = self.width_pct {
            let width = format!("{pct}%");
            cell = cell
                .attr(AttrName::Width, AttrValue::Text(width.clone()))
                .style(prop("width"), value(&width));
        }

        if self.padding_x_px > 0 || self.padding_y_px > 0 {
            let padding = format!("{} {}", px(self.padding_y_px), px(self.padding_x_px));
            cell = styled(cell, &[("padding", &padding)]);
        }

        if let Some(color) = &self.background {
            cell = cell
                .attr(
                    AttrName::Bgcolor,
                    AttrValue::Text(color.as_str().to_owned()),
                )
                .style(prop("background-color"), color.style_value());
        }

        if stacking {
            cell = cell.class(ClassName::new(STACK_CLASS).expect("literal"));
        }

        cell
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Row;
    use crate::markup::{MediaQuery, Stylesheet};
    use crate::render::render;

    /// The attribute is for Outlook, the declaration for everyone else, the
    /// same pairing Container relies on.
    #[test]
    fn width_is_both_an_attribute_and_a_declaration() {
        let html = render(&[Row::new().column(Column::new().width(33)).build()]);

        assert!(html.contains(r#"width="33%""#), "{html}");
        assert!(html.contains("width:33%"), "{html}");
    }

    #[test]
    fn stack_rules_target_that_class_and_are_important() {
        let css = Stylesheet::new()
            .media(MediaQuery::MaxWidth(600), stack_rules())
            .to_css();

        assert!(css.contains(".fm-stack{"), "{css}");
        assert!(css.contains("display:block!important"), "{css}");
        assert!(css.contains("width:100%!important"), "{css}");
    }

    #[test]
    fn columns_default_to_top_aligned() {
        let html = render(&[Row::new().column(Column::new()).build()]);
        assert!(html.contains(r#"valign="top""#), "{html}");

        let middle = render(&[Row::new().column(Column::new().valign_middle()).build()]);
        assert!(middle.contains(r#"valign="middle""#), "{middle}");
    }

    #[test]
    fn padding_is_omitted_when_zero() {
        let html = render(&[Row::new().column(Column::new()).build()]);
        assert!(!html.contains("padding:"), "{html}");

        let padded = render(&[Row::new()
            .column(Column::new().padding_x(16).padding_y(24))
            .build()]);
        assert!(padded.contains("padding:24px 16px"), "{padded}");
    }
}
