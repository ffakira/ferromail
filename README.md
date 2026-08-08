# ferromail

[![crates.io](https://img.shields.io/crates/v/ferromail.svg)](https://crates.io/crates/ferrom
ail)
[![CI](https://github.com/ffakira/ferromail/actions/workflows/ci.yml/badge.svg)](https://gith
ub.com/ffakira/ferromail/actions/workflows/ci.yml)

Type-safe HTML email component builder for Rust.

Email HTML is its own dialect: table layouts, inline styles, Outlook conditional
comments. ferromail builds a markup tree you can't accidentally break, then
renders it to something mail clients actually display.

No runtime dependencies.

## Status

Proof of concept. The markup tree and renderer work and are property tested, but
there are no components yet and no document scaffolding, so this is not useful
for sending real email. The API will change.

## Example

```rust
use ferromail::markup::{Element, Node, Property, StyleValue, Tag, Url, UrlAttr};
use ferromail::render::render;

let button = Element::new(Tag::Td)
    .style(Property::new("background").unwrap(), StyleValue::parse("#2563eb").unwrap())
    .child(Node::Element(
        Element::new(Tag::A)
            .url_attr(UrlAttr::Href, Url::parse("https://example.com").unwrap())
            .text("View order <script>")
    ));

println!("{}", render(&[Node::Element(button)]));

<td style="background:#2563eb"><a href="https://example.com">View order
&lt;script&gt;
```

**What is enforced**

Untrusted strings cannot become markup. Text is escaped on render, and every
value that reaches an attribute goes through a type that rejects anything able
to close it:

- `Url` accepts `http`, `https`, `mailto` and `tel` only, so `javascript:` has no path into `href` or `src`
- `StyleValue` rejects CSS functions, comments and backslash escapes, so `expression()` and its encoded variants cannot appear in a `style` attribute
- `RawHtml::trusted` is the single unescaped entry point, and it is named so it shows up in review

Outlook conditional comments are typed rather than strings, so `!mso` emits the downlevel-revealed syntax instead of silently hiding its content.

**Not done yet**

Components, document scaffolding (doctype, head, media queries), and Tailwind behind a feature flag.

## License

MIT or Apache-2.0, at your option.
