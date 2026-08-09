//! Prebuilt email components.

use crate::markup::{Element, Property, StyleValue};

pub mod button;
pub mod container;
pub mod document;

pub use button::Button;
pub use container::Container;
pub use document::Document;

/// Interns a property name written as a literal in this crate.
///
/// # Panics
///
/// If `name` is not a valid property. These are `pub(crate)` helpers called
/// only with literals, so a panic is a typo caught by our own tests. No
/// consumer input can reach it.
pub(crate) fn prop(name: &str) -> Property {
    Property::new(name).expect("valid property literal")
}

/// Builds one declaration from literals the crate controls.
///
/// # Panics
///
/// If the property or value is not valid. See [`prop`].
pub(crate) fn decl(property: &str, value: &str) -> (Property, StyleValue) {
    (
        prop(property),
        StyleValue::parse(value).expect("valid style literal"),
    )
}

/// Applies declarations in order. Later entries overwrite earlier ones in place.
///
/// Only for values this crate generates. Anything a caller supplied should
/// already be a validated [`StyleValue`] and go through [`Element::style`].
pub(crate) fn styled(el: Element, decls: &[(&str, &str)]) -> Element {
    decls.iter().fold(el, |el, (property, value)| {
        let (p, v) = decl(property, value);
        el.style(p, v)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::{Node, Tag};
    use crate::render::render;

    #[test]
    fn applies_in_order() {
        let el = styled(
            Element::new(Tag::Td),
            &[
                ("background", "#2563eb"),
                ("padding", "12px 24px"),
                ("border-radius", "4px"),
            ],
        );

        assert_eq!(
            render(&[Node::Element(el)]),
            r#"<td style="background:#2563eb;padding:12px 24px;border-radius:4px"></td>"#
        );
    }

    #[test]
    fn later_declaration_overwrites_in_place() {
        let el = styled(
            Element::new(Tag::Td),
            &[("color", "#111"), ("width", "100%"), ("color", "#222")],
        );

        assert_eq!(
            render(&[Node::Element(el)]),
            r#"<td style="color:#222;width:100%"></td>"#
        );
    }
}
