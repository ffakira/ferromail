//! A typed stylesheet, for the `<style>` block an email needs to carry
//! media queries.
//!
//! A `<style>` block is raw CSS text, which is the one thing this crate exists
//! to avoid emitting. So nothing here takes a string: selectors are
//! [`ClassName`], properties are [`Property`], values are [`StyleValue`], and
//! the braces, colons and `@media` come from the renderer. `</style>` is
//! therefore unrepresentable, because every component type already rejects `<` and
//! `>`.
//!
//! # Media queries are an enhancement, not the mechanism
//!
//! Gmail's app strips `<style>` when the account is not a Gmail one, and
//! Outlook's Word engine ignores media queries entirely. A layout that is
//! responsive *only* through breakpoints is broken in both. Build a fluid base
//! with `max-width` and percentage widths, then use this to refine it.

use std::fmt::Write as _;

use super::{ClassName, Property, StyleMap, StyleValue};

/// What a rule applies to.
///
/// Classes only. Element and id selectors are unreliable across clients, and a
/// class is the one thing an email can attach and target predictably.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Selector {
    Class(ClassName),
}

impl Selector {
    pub fn write(&self, out: &mut String) {
        match self {
            Selector::Class(name) => {
                out.push('.');
                out.push_str(name.as_str());
            }
        }
    }
}

/// A viewport condition.
///
/// `only screen` is included because some older clients apply the block
/// regardless of media type otherwise.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaQuery {
    MaxWidth(u32),
    MinWidth(u32),
}

impl MediaQuery {
    pub fn write(&self, out: &mut String) {
        let (feature, px) = match *self {
            MediaQuery::MaxWidth(px) => ("max-width", px),
            MediaQuery::MinWidth(px) => ("min-width", px),
        };
        write!(out, "only screen and ({feature}:{px}px)").expect("infallible");
    }
}

/// One selector and the declarations it applies.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rule {
    selector: Selector,
    decls: StyleMap,
    important: bool,
}

impl Rule {
    pub fn class(name: ClassName) -> Self {
        Self {
            selector: Selector::Class(name),
            decls: StyleMap::new(),
            important: false,
        }
    }

    #[must_use]
    pub fn set(mut self, prop: Property, value: StyleValue) -> Self {
        self.decls.set(prop, value);
        self
    }

    /// Appends `!important` to every declaration in this rule.
    ///
    /// Webmail clients inject their own stylesheets, so a responsive override
    /// that is not `!important` frequently loses. A value that already ends in
    /// `!important` is left alone rather than doubled, since
    /// `display:block!important!important` is invalid and gets dropped whole.
    #[must_use]
    pub fn important(mut self) -> Self {
        self.important = true;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.decls.is_empty()
    }

    fn write(&self, out: &mut String) {
        if self.is_empty() {
            return;
        }

        self.selector.write(out);
        out.push('{');

        for (i, (prop, value)) in self.decls.declarations().enumerate() {
            if i > 0 {
                out.push(';');
            }
            out.push_str(prop.as_str());
            out.push(':');
            out.push_str(value.as_str());

            if self.important && !value.as_str().contains("!important") {
                out.push_str("!important");
            }
        }

        out.push('}');
    }
}

/// The contents of a `<style>` block.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Stylesheet {
    rules: Vec<Rule>,
    media: Vec<(MediaQuery, Vec<Rule>)>,
}

impl Stylesheet {
    pub fn new() -> Self {
        Self::default()
    }

    /// A rule that applies at every width.
    #[must_use]
    pub fn rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Rules that apply only when the query matches.
    #[must_use]
    pub fn media(mut self, query: MediaQuery, rules: impl IntoIterator<Item = Rule>) -> Self {
        self.media.push((query, rules.into_iter().collect()));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.rules.iter().all(Rule::is_empty)
            && self
                .media
                .iter()
                .all(|(_, rules)| rules.iter().all(Rule::is_empty))
    }

    /// Serialises to CSS, minified.
    ///
    /// No spaces or newlines: Gmail clips a message over 102KB and hides
    /// everything after the cut, so bytes in the head are bytes not spent on
    /// content.
    pub fn to_css(&self) -> String {
        let mut out = String::new();

        for rule in &self.rules {
            rule.write(&mut out);
        }

        for (query, rules) in &self.media {
            if rules.iter().all(Rule::is_empty) {
                continue;
            }

            out.push_str("@media ");
            query.write(&mut out);
            out.push('{');
            for rule in rules {
                rule.write(&mut out);
            }
            out.push('}');
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(name: &str) -> ClassName {
        ClassName::new(name).expect("valid class")
    }

    fn prop(name: &str) -> Property {
        Property::new(name).expect("valid property")
    }

    fn value(raw: &str) -> StyleValue {
        StyleValue::parse(raw).expect("valid value")
    }

    #[test]
    fn renders_a_media_block() {
        let sheet = Stylesheet::new().media(
            MediaQuery::MaxWidth(600),
            [
                Rule::class(class("stack"))
                    .set(prop("display"), value("block"))
                    .set(prop("width"), value("100%"))
                    .important(),
                Rule::class(class("hide-sm"))
                    .set(prop("display"), value("none"))
                    .important(),
            ],
        );

        assert_eq!(
            sheet.to_css(),
            "@media only screen and (max-width:600px)\
             {.stack{display:block!important;width:100%!important}\
             .hide-sm{display:none!important}}"
        );
    }

    #[test]
    fn top_level_rules_come_before_media() {
        let sheet = Stylesheet::new()
            .rule(Rule::class(class("body")).set(prop("margin"), value("0")))
            .media(
                MediaQuery::MinWidth(601),
                [Rule::class(class("wide")).set(prop("width"), value("600px"))],
            );

        assert_eq!(
            sheet.to_css(),
            ".body{margin:0}@media only screen and (min-width:601px){.wide{width:600px}}"
        );
    }

    #[test]
    fn important_is_not_doubled() {
        let sheet = Stylesheet::new().rule(
            Rule::class(class("x"))
                .set(prop("display"), value("block !important"))
                .important(),
        );

        assert_eq!(sheet.to_css(), ".x{display:block !important}");
    }

    #[test]
    fn empty_rules_and_blocks_are_skipped() {
        let sheet = Stylesheet::new().rule(Rule::class(class("nothing"))).media(
            MediaQuery::MaxWidth(600),
            [Rule::class(class("also-nothing"))],
        );

        assert!(sheet.is_empty());
        assert_eq!(sheet.to_css(), "");
    }

    /// The reason this type exists rather than a raw CSS string.
    #[test]
    fn a_style_block_cannot_be_closed_from_inside() {
        assert!(ClassName::new("x</style><script>").is_none());
        assert!(Property::new("x</style>").is_none());
        assert!(StyleValue::parse("red</style><script>alert(1)</script>").is_err());
    }
}
