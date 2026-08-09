//! Bulletproof call-to-action button.

use super::{prop, styled};
use crate::markup::{
    AttrName, AttrValue, Color, Condition, Element, Node, StyleValue, Tag, Url, UrlAttr,
};

/// A bulletproof call-to-action button.
///
/// Emits two siblings: a VML `v:roundrect` inside an `mso` conditional for
/// Outlook's Word renderer, and a styled `<a>` inside a `!mso` conditional
/// for everyone else. Exactly one is ever shown.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Button {
    href: Url,
    label: String,
    background: Color,
    color: Color,
    font_family: StyleValue,
    font_size_px: u32,
    width_px: u32,
    height_px: u32,
    radius_px: u32,
}

impl Button {
    /// # Panics
    ///
    /// Never in practice: the defaults are crate literals, checked by tests.
    pub fn new(href: Url, label: impl Into<String>) -> Self {
        Self {
            href,
            label: label.into(),
            background: Color::hex("#2563eb").expect("literal"),
            color: Color::hex("#ffffff").expect("literal"),
            font_family: StyleValue::parse("Arial, sans-serif").expect("literal"),
            font_size_px: 16,
            width_px: 220,
            height_px: 44,
            radius_px: 4,
        }
    }

    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    #[must_use]
    pub fn font_family(mut self, family: StyleValue) -> Self {
        self.font_family = family;
        self
    }

    #[must_use]
    pub fn font_size(mut self, px: u32) -> Self {
        self.font_size_px = px;
        self
    }

    #[must_use]
    pub fn size(mut self, width_px: u32, height_px: u32) -> Self {
        self.width_px = width_px;
        self.height_px = height_px;
        self
    }

    #[must_use]
    pub fn radius(mut self, px: u32) -> Self {
        self.radius_px = px;
        self
    }

    /// VML has no `border-radius`; it takes the corner as a percentage of the
    /// shorter side, capped at the 50% that makes a pill.
    fn arcsize(&self) -> String {
        // checked_div rather than guarding on height first: a zero height is
        // degenerate anyway, and 0% is the right answer for it.
        let pct = self
            .radius_px
            .saturating_mul(100)
            .checked_div(self.height_px)
            .unwrap_or(0)
            .min(50);

        format!("{pct}%")
    }

    /// Renders the button as a pair of conditional nodes.
    ///
    /// Infallible: colours and the font stack were validated when they were
    /// set, and every other value is generated from a `u32`.
    pub fn build(&self) -> Vec<Node> {
        let width = format!("{}px", self.width_px);
        let height = format!("{}px", self.height_px);
        let radius = format!("{}px", self.radius_px);
        let font_size = format!("{}px", self.font_size_px);

        // Outlook: VML shape, text centred by v-text-anchor. w:anchorlock
        // stops Word turning the label into an editable field.
        let label = styled(
            Element::new(Tag::Center).text(self.label.clone()),
            &[("font-size", &font_size), ("font-weight", "bold")],
        )
        .style(prop("color"), self.color.style_value())
        .style(prop("font-family"), self.font_family.clone());

        let vml = styled(
            Element::new(Tag::VRoundRect)
                .url_attr(UrlAttr::Href, self.href.clone())
                .attr(AttrName::ArcSize, AttrValue::Text(self.arcsize()))
                .attr(
                    AttrName::FillColor,
                    AttrValue::Text(self.background.as_str().to_owned()),
                )
                .attr(
                    AttrName::StrokeColor,
                    AttrValue::Text(self.background.as_str().to_owned()),
                ),
            &[
                ("height", &height),
                ("width", &width),
                ("v-text-anchor", "middle"),
            ],
        )
        .child(Node::Element(Element::new(Tag::WAnchorLock)))
        .child(Node::Element(label));

        // Everyone else. line-height equal to height centres the label
        // vertically without padding, which Outlook.com and Gmail both honour.
        let anchor = styled(
            Element::new(Tag::A)
                .url_attr(UrlAttr::Href, self.href.clone())
                .text(self.label.clone()),
            &[
                ("display", "inline-block"),
                ("font-size", &font_size),
                ("font-weight", "bold"),
                ("line-height", &height),
                ("text-align", "center"),
                ("text-decoration", "none"),
                ("width", &width),
            ],
        )
        .style(prop("color"), self.color.style_value())
        .style(prop("font-family"), self.font_family.clone());

        // The colour goes on a `td`, not the anchor: several webmail clients
        // drop background and padding on inline elements. `bgcolor` repeats
        // the declaration because some honour the attribute and ignore the
        // CSS, and others do the reverse.
        let cell = styled(
            Element::new(Tag::Td)
                .attr(AttrName::Align, AttrValue::Text("center".to_owned()))
                .attr(
                    AttrName::Bgcolor,
                    AttrValue::Text(self.background.as_str().to_owned()),
                ),
            &[("border-radius", &radius)],
        )
        .style(prop("background-color"), self.background.style_value())
        .child(Node::Element(anchor));

        // role=presentation stops screen readers announcing a layout table as
        // data. border/cellpadding/cellspacing are the attribute forms, which
        // Outlook honours where the CSS equivalents are ignored.
        let table = styled(
            Element::new(Tag::Table)
                .attr(AttrName::Role, AttrValue::Text("presentation".to_owned()))
                .attr(AttrName::Border, AttrValue::Int(0))
                .attr(AttrName::Cellpadding, AttrValue::Int(0))
                .attr(AttrName::Cellspacing, AttrValue::Int(0)),
            &[("border-collapse", "collapse")],
        )
        .child(Node::Element(
            Element::new(Tag::Tr).child(Node::Element(cell)),
        ));

        vec![
            Node::Conditional {
                cond: Condition::Mso,
                children: vec![Node::Element(vml)],
            },
            Node::Conditional {
                cond: Condition::NotMso,
                children: vec![Node::Element(table)],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::render;

    #[test]
    fn renders_both_branches() {
        let href = Url::parse("https://example.com/confirm?a=1&b=2").expect("valid");
        let html = render(&Button::new(href, "Confirm").build());

        // Outlook branch: downlevel-hidden, so non-Outlook clients skip it.
        assert!(html.starts_with("<!--[if mso]>"), "{html}");
        assert!(html.contains("<v:roundrect"), "{html}");
        assert!(html.contains("<w:anchorlock />"), "{html}");
        assert!(html.contains("arcsize=\"9%\""), "{html}");

        // Fallback branch: downlevel-revealed, table-wrapped.
        assert!(html.contains("<!--[if !mso]><!-->"), "{html}");
        assert!(html.ends_with("<!--<![endif]-->"), "{html}");
        assert!(html.contains(r#"<table role="presentation""#), "{html}");
        assert!(
            html.contains("<a href=\"https://example.com/confirm?a=1&amp;b=2\""),
            "{html}"
        );

        // The colour is stated twice: attribute for clients that ignore the
        // declaration, declaration for clients that ignore the attribute.
        assert!(html.contains(r##"bgcolor="#2563eb""##), "{html}");
        assert!(html.contains("background-color:#2563eb"), "{html}");

        // The label appears once per branch, never unescaped.
        assert_eq!(html.matches("Confirm").count(), 2, "{html}");
    }

    /// A hostile colour cannot reach `Button` at all now: `background` takes a
    /// `Color`, and `Color::hex` is the only constructor.
    #[test]
    fn a_hostile_colour_is_not_a_colour() {
        assert!(Color::hex("#fff; } </style><script>alert(1)</script>").is_none());
        assert!(Color::hex("red").is_none());
        assert!(Color::hex("#12345").is_none());
        assert!(Color::hex("#GGG").is_none());

        assert_eq!(
            Color::hex("#2563EB").expect("valid").as_str(),
            "#2563eb",
            "hex should normalise to lowercase"
        );
    }

    #[test]
    fn arcsize_of_a_zero_height_button_is_zero() {
        let href = Url::parse("https://example.com").expect("valid");
        let html = render(&Button::new(href, "x").size(200, 0).radius(8).build());

        assert!(html.contains(r#"arcsize="0%""#), "{html}");
    }

    #[test]
    fn arcsize_is_capped_at_a_pill() {
        let href = Url::parse("https://example.com").expect("valid");
        let html = render(&Button::new(href, "x").size(200, 40).radius(999).build());

        assert!(html.contains("arcsize=\"50%\""), "{html}");
    }
}
