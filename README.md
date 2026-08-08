# ferromail

Type-safe email component builder for Rust.

Email HTML is its own dialect: table layouts, inline styles, Outlook conditional
comments. ferromail builds a markup tree you can't accidentally break, then
renders it to something mail clients actually display.

## Status

Early. The API does not exist yet, so don't depend on this.

## Design

Components build a `markup` tree. Text escapes on render, and raw HTML needs an
explicit constructor, so injection has exactly one auditable entry point. A
renderer turns the tree into HTML.

Tailwind support is planned behind a `tailwind` feature flag. Utility classes
resolve to inline styles at build time, with media queries kept in a `<style>`
block since they can't be inlined.
