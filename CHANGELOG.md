# Changelog

Notable changes to ferromail. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the major version is 0 the API is unstable, and breaking changes ship in
patch releases. They are marked **Breaking** below.

## [Unreleased] 0.0.3

### Added

- `components::Container`, a centered column that fills a narrow viewport and
  stops growing on a wide one. The inner table carries both a `width` attribute
  and a `max-width` declaration: Outlook honours the attribute and ignores the
  declaration, every other client does the reverse, so the column is fluid
  without a conditional comment. Padding sits on the content cell, inside the
  column, so it eats into the width rather than adding to it.

  This answers the worst finding in the client-support report: `<body>` is
  unsupported in 16 clients, so width, background and centering cannot live
  there.

Planned: Tailwind-style utilities, a divider for spacing between siblings since
Outlook ignores margin, and rendering checked against a real Outlook rather than
reasoned about.

## [0.0.2] 2026-08-09

The first release with anything you can build an email out of. 0.0.1 was
markup primitives only.

### Added

- `components::Document`, an `<html>` wrapper that declares the VML namespaces.
  Without them the VML in `Button` does not render at all, so 0.0.1 could not
  have produced a working Outlook button even by hand.
- `components::Button`, a call to action with a VML `v:roundrect` for Outlook
  and a table fallback for everyone else, with the colour on a `td` rather than
  on an inline-block anchor. Measured with `html-check`, putting it on the `td`
  moved the bare button from 34.5% supported and 54.2% partial to 86.9% and
  10.8%.
- `markup::Stylesheet`, a typed `<style>` block with `@media` support. Typed
  down to the selector, so `</style>` cannot be written from inside it.
- `markup::Color`, a validated hex colour. Being the only constructor is what
  lets `Button::build` return nodes rather than a `Result`.
- The `html!` macro. Syntax only: it expands to the same builder calls, so it
  opens no path the API does not already allow. Unknown tags and attributes are
  a compile error that names the offender.
- `Node::Style`, `Tag::{Html, Head, Body, Meta, Title, Style, VRoundRect,
  WAnchorLock}`, and the attributes those need.
- `From` conversions on `AttrValue` for `&str`, `String`, `u32` and `Url`.
- `rust-version = "1.85"`, now enforced by CI rather than asserted.
- A client-support report generated from Mailpit's `html-check`, run against
  fixtures through a local SMTP catcher.

### Changed

- **Breaking.** Public enums are `#[non_exhaustive]`: `Tag`, `AttrName`,
  `UrlAttr`, `AttrValue`, `Node`, `Condition`, `MediaQuery`, `Selector`,
  `UrlError`, `StyleValueError`. A downstream exhaustive `match` on any of them
  now needs a wildcard arm. This is deliberate, and done before anyone depends
  on the crate: every future tag and attribute is now an additive change rather
  than a breaking one.
- `markup` was a single module and is now a directory, with `attr`, `style`,
  `tag`, `tree` and `url` submodules. Paths such as `ferromail::markup::Url`
  are unchanged, because everything is re-exported.

### Fixed

- `arcsize` used a manual zero check before dividing, which
  `clippy::manual_checked_ops` flags on Rust 1.97.

## [0.0.1] 2026-08-08

Markup primitives and the renderer. Not useful for sending email: there were no
components and no document scaffolding.

### Added

- `markup::Node`, `Element`, `Tag`, `AttrName`, `AttrValue` and `RawHtml`.
- `markup::Url` behind a scheme allowlist of `http`, `https`, `mailto` and
  `tel`.
- `markup::StyleValue`, rejecting CSS functions, comments and backslash
  escapes.
- `markup::ClassName`, `Property` and `StyleMap`.
- `markup::Condition`, so Outlook conditional comments are typed rather than
  strings and `!mso` emits the downlevel-revealed form.
- `render::render`, escaping text and attribute values, with a property test
  suite over deliberately hostile input.
- `UrlAttr`, disjoint from `AttrName`, so an unparsed string cannot reach
  `href` or `src`. `Element`'s fields are private and `StyleMap` holds
  `StyleValue` rather than `String`, so neither the builder nor the value
  checks can be sidestepped.

[Unreleased]: https://github.com/ffakira/ferromail/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/ffakira/ferromail/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/ffakira/ferromail/releases/tag/v0.0.1
