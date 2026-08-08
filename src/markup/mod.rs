//! The validated markup primitives every component is built from.
//!
//! Each type here is a gate: [`Url`] holds only allowlisted schemes,
//! [`StyleValue`] only declaration values that cannot escape a `style`
//! attribute, [`UrlAttr`] keeps unvalidated strings out of `href`. Higher
//! layers compose these; they never re-implement the checks.

pub mod attr;
pub mod style;
pub mod tag;
pub mod tree;
pub mod url;

pub use attr::{AttrName, AttrValue, UrlAttr};
pub use style::{
    ClassName, Color, MediaQuery, Property, Rule, Selector, StyleMap, StyleValue, StyleValueError,
    Stylesheet,
};
pub use tag::Tag;
pub use tree::{Condition, Element, Node, RawHtml};
pub use url::{Url, UrlError};
