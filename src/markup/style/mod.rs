//! Validated CSS primitives.
//!
//! Split into a folder because this is the layer a utility vocabulary
//! (Tailwind-style tokens, colour and spacing scales) will grow on top of.
//! The types here stay small and total; the vocabulary lives above them.

pub mod class;
pub mod color;
pub mod map;
pub mod property;
pub mod sheet;
pub mod value;

pub use class::ClassName;
pub use color::Color;
pub use map::StyleMap;
pub use property::Property;
pub use sheet::{MediaQuery, Rule, Selector, Stylesheet};
pub use value::{StyleValue, StyleValueError};
