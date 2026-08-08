//! An ordered set of declarations, one per property.

use super::{Property, StyleValue};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StyleMap {
    decls: Vec<(Property, StyleValue)>,
}

impl StyleMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, prop: Property, value: StyleValue) {
        match self.decls.iter_mut().find(|(p, _)| *p == prop) {
            Some((_, slot)) => *slot = value,
            None => self.decls.push((prop, value)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.decls.is_empty()
    }

    pub fn get(&self, prop: &Property) -> Option<&StyleValue> {
        self.decls.iter().find(|(p, _)| p == prop).map(|(_, v)| v)
    }

    pub fn declarations(&self) -> impl Iterator<Item = (&Property, &StyleValue)> {
        self.decls.iter().map(|(p, v)| (p, v))
    }

    pub fn fill_from(&mut self, other: &StyleMap) {
        for (prop, value) in &other.decls {
            if self.get(prop).is_none() {
                self.decls.push((prop.clone(), value.clone()));
            }
        }
    }
}
