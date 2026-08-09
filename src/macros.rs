//! `html!`, a thin syntax layer over [`Element`](crate::markup::Element).
//!
//! The macro only rearranges calls you could write by hand. Every value still
//! goes through the same validated types, so nothing here can bypass a check:
//! `href` and `src` demand a [`Url`](crate::markup::Url), `class` demands a
//! [`ClassName`](crate::markup::ClassName), and text is escaped on render
//! exactly as it would be otherwise.
//!
//! ```rust
//! use ferromail::html;
//! use ferromail::markup::Url;
//! use ferromail::render::render;
//!
//! let url = Url::parse("https://example.com").expect("valid");
//! let nodes = html! {
//!     table border=(0) cellpadding=(0) {
//!         tr {
//!             td align=("center") {
//!                 a href=(url) { "View order" }
//!             }
//!         }
//!     }
//! };
//!
//! assert_eq!(
//!     render(&nodes),
//!     concat!(
//!         r#"<table border="0" cellpadding="0"><tr><td align="center">"#,
//!         r#"<a href="https://example.com">View order</a>"#,
//!         "</td></tr></table>",
//!     )
//! );
//! ```
//!
//! # Syntax
//!
//! - `tag { .. }` is an element with children
//! - `tag;` is an element with none, which is also the form void tags take
//! - `name=(expr)` is an attribute; the parentheses are always required
//! - `"literal"` is escaped text
//! - `(expr)` is escaped text from a value that is `Into<String>`
//! - `{{ name }}` is a placeholder, filled by
//!   [`render_with`](crate::render::render_with)
//! - `@(expr)` splices one node, so a component composes into a tree
//! - `@[expr]` splices many, which is what [`Button::build`] and
//!   [`Document::build`] return
//!
//! [`Button::build`]: crate::components::Button::build
//! [`Document::build`]: crate::components::Document::build
//!
//! # What it refuses
//!
//! The tag and attribute tables are closed, so anything outside them is a
//! compile error naming the offender rather than a matcher failure.
//!
//! ```compile_fail
//! # use ferromail::html;
//! let _ = html! { marquee { "no such tag" } };
//! ```
//!
//! Event handlers have no variant and never will:
//!
//! ```compile_fail
//! # use ferromail::html;
//! let _ = html! { td onclick=("alert(1)") { "no" } };
//! ```
//!
//! A value must be parenthesised:
//!
//! ```compile_fail
//! # use ferromail::html;
//! let _ = html! { td align="center" { "no" } };
//! ```
//!
//! And `href` will not take a string, only a parsed `Url`:
//!
//! ```compile_fail
//! # use ferromail::html;
//! let _ = html! { a href=("javascript:alert(1)") { "no" } };
//! ```
//!
//! Attributes are matched by name, so `href` and `src` route to
//! [`Element::url_attr`](crate::markup::Element::url_attr) and `class` to
//! [`Element::class`](crate::markup::Element::class). Everything else goes
//! through [`Element::attr`](crate::markup::Element::attr) with the value
//! converted by `Into<AttrValue>`.

/// Builds a `Vec<Node>` from an HTML-like tree.
///
/// See the [module docs](self) for the syntax.
#[macro_export]
macro_rules! html {
    ($($tt:tt)*) => {{
        let mut __nodes: ::std::vec::Vec<$crate::markup::Node> = ::std::vec::Vec::new();
        let __sink = &mut __nodes;
        $crate::__html_nodes!(__sink; $($tt)*);
        __nodes
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __html_nodes {
    ($nodes:ident;) => {};

    // string literal becomes escaped text
    ($nodes:ident; $text:literal $($rest:tt)*) => {
        $nodes.push($crate::markup::Node::Text(
            ::std::convert::Into::into($text),
        ));
        $crate::__html_nodes!($nodes; $($rest)*);
    };

    // parenthesised expression becomes escaped text
    ($nodes:ident; ($text:expr) $($rest:tt)*) => {
        $nodes.push($crate::markup::Node::Text(
            ::std::convert::Into::into($text),
        ));
        $crate::__html_nodes!($nodes; $($rest)*);
    };

    // @(expr) splices one node, so a component drops into a tree.
    ($nodes:ident; @ ( $node:expr ) $($rest:tt)*) => {
        $nodes.push(::std::convert::Into::<$crate::markup::Node>::into($node));
        $crate::__html_nodes!($nodes; $($rest)*);
    };

    // @[expr] splices many, which is what Button::build and Document::build
    // return.
    ($nodes:ident; @ [ $nodes_expr:expr ] $($rest:tt)*) => {
        ::std::iter::Extend::extend(
            $nodes,
            ::std::iter::IntoIterator::into_iter($nodes_expr),
        );
        $crate::__html_nodes!($nodes; $($rest)*);
    };

    // {{ name }} is a placeholder, filled by render_with. A Rust identifier is
    // always a valid VarName, so the expect cannot fire for anything the
    // macro will accept.
    ($nodes:ident; { { $name:ident } } $($rest:tt)*) => {
        $nodes.push($crate::markup::Node::Var(
            $crate::markup::VarName::new(::std::stringify!($name))
                .expect("a Rust identifier is a valid VarName"),
        ));
        $crate::__html_nodes!($nodes; $($rest)*);
    };

    // element with a body
    ($nodes:ident; $tag:ident $($name:ident = ($value:expr))* { $($body:tt)* } $($rest:tt)*) => {
        {
            let __el = $crate::markup::Element::new($crate::__html_tag!($tag));
            $(let __el = $crate::__html_attr!(__el, $name, $value);)*
            let __el = $crate::html!($($body)*)
                .into_iter()
                .fold(__el, $crate::markup::Element::child);
            $nodes.push($crate::markup::Node::Element(__el));
        }
        $crate::__html_nodes!($nodes; $($rest)*);
    };

    // element with no body, terminated by a semicolon
    ($nodes:ident; $tag:ident $($name:ident = ($value:expr))* ; $($rest:tt)*) => {
        {
            let __el = $crate::markup::Element::new($crate::__html_tag!($tag));
            $(let __el = $crate::__html_attr!(__el, $name, $value);)*
            $nodes.push($crate::markup::Node::Element(__el));
        }
        $crate::__html_nodes!($nodes; $($rest)*);
    };

    // Must stay last. Without it a syntax slip reports the arm the matcher
    // happened to give up on, which points somewhere unrelated: forgetting
    // the parentheses on a value used to say "while trying to match `alt`".
    ($nodes:ident; $($rest:tt)+) => {
        ::std::compile_error!(::std::concat!(
            "html!: could not parse `",
            ::std::stringify!($($rest)+),
            "`. Expected `tag { .. }`, `tag;`, `name=(value)` with the \
             parentheses, a string literal, or `(expr)` for interpolated text."
        ));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __html_tag {
    (html) => {
        $crate::markup::Tag::Html
    };
    (head) => {
        $crate::markup::Tag::Head
    };
    (body) => {
        $crate::markup::Tag::Body
    };
    (meta) => {
        $crate::markup::Tag::Meta
    };
    (title) => {
        $crate::markup::Tag::Title
    };
    (table) => {
        $crate::markup::Tag::Table
    };
    (tbody) => {
        $crate::markup::Tag::TBody
    };
    (tr) => {
        $crate::markup::Tag::Tr
    };
    (td) => {
        $crate::markup::Tag::Td
    };
    (div) => {
        $crate::markup::Tag::Div
    };
    (center) => {
        $crate::markup::Tag::Center
    };
    (p) => {
        $crate::markup::Tag::P
    };
    (span) => {
        $crate::markup::Tag::Span
    };
    (strong) => {
        $crate::markup::Tag::Strong
    };
    (em) => {
        $crate::markup::Tag::Em
    };
    (a) => {
        $crate::markup::Tag::A
    };
    (br) => {
        $crate::markup::Tag::Br
    };
    (h1) => {
        $crate::markup::Tag::H1
    };
    (h2) => {
        $crate::markup::Tag::H2
    };
    (h3) => {
        $crate::markup::Tag::H3
    };
    (h4) => {
        $crate::markup::Tag::H4
    };
    (h5) => {
        $crate::markup::Tag::H5
    };
    (h6) => {
        $crate::markup::Tag::H6
    };
    (ul) => {
        $crate::markup::Tag::Ul
    };
    (ol) => {
        $crate::markup::Tag::Ol
    };
    (li) => {
        $crate::markup::Tag::Li
    };
    (img) => {
        $crate::markup::Tag::Img
    };

    // Must stay last: macro_rules tries arms in order.
    ($other:ident) => {
        ::std::compile_error!(::std::concat!(
            "html!: unknown tag `",
            ::std::stringify!($other),
            "`. ferromail emits a closed set of tags, so a typo and an \
             unsupported element look the same here. If it genuinely belongs \
             in email, add a variant to markup::Tag and an arm to __html_tag!."
        ))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __html_attr {
    ($el:expr, href, $value:expr) => {
        $el.url_attr($crate::markup::UrlAttr::Href, $value)
    };
    ($el:expr, src, $value:expr) => {
        $el.url_attr($crate::markup::UrlAttr::Src, $value)
    };
    ($el:expr, class, $value:expr) => {
        $el.class($value)
    };
    ($el:expr, $name:ident, $value:expr) => {
        $el.attr(
            $crate::__html_attr_name!($name),
            ::std::convert::Into::<$crate::markup::AttrValue>::into($value),
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __html_attr_name {
    (alt) => {
        $crate::markup::AttrName::Alt
    };
    (title) => {
        $crate::markup::AttrName::Title
    };
    (target) => {
        $crate::markup::AttrName::Target
    };
    (width) => {
        $crate::markup::AttrName::Width
    };
    (height) => {
        $crate::markup::AttrName::Height
    };
    (align) => {
        $crate::markup::AttrName::Align
    };
    (valign) => {
        $crate::markup::AttrName::Valign
    };
    (bgcolor) => {
        $crate::markup::AttrName::Bgcolor
    };
    (border) => {
        $crate::markup::AttrName::Border
    };
    (cellpadding) => {
        $crate::markup::AttrName::Cellpadding
    };
    (cellspacing) => {
        $crate::markup::AttrName::Cellspacing
    };
    (colspan) => {
        $crate::markup::AttrName::Colspan
    };
    (rowspan) => {
        $crate::markup::AttrName::Rowspan
    };
    (id) => {
        $crate::markup::AttrName::Id
    };
    (role) => {
        $crate::markup::AttrName::Role
    };
    (dir) => {
        $crate::markup::AttrName::Dir
    };
    (lang) => {
        $crate::markup::AttrName::Lang
    };
    (charset) => {
        $crate::markup::AttrName::Charset
    };
    (content) => {
        $crate::markup::AttrName::Content
    };
    (name) => {
        $crate::markup::AttrName::Name
    };
    (arcsize) => {
        $crate::markup::AttrName::ArcSize
    };
    (fillcolor) => {
        $crate::markup::AttrName::FillColor
    };
    (strokecolor) => {
        $crate::markup::AttrName::StrokeColor
    };

    // Must stay last: macro_rules tries arms in order.
    ($other:ident) => {
        ::std::compile_error!(::std::concat!(
            "html!: unknown attribute `",
            ::std::stringify!($other),
            "`. Event handlers such as onclick have no variant by design and \
             never will. For anything else, add a variant to markup::AttrName \
             and an arm to __html_attr_name!. Note href and src are not here: \
             they take a parsed Url through url_attr."
        ))
    };
}

#[cfg(test)]
mod tests {
    use crate::markup::{ClassName, Element, Tag};
    use crate::render::render;

    fn url() -> crate::markup::Url {
        crate::markup::Url::parse("https://example.com/order").expect("valid")
    }

    #[test]
    fn nests_and_escapes() {
        let nodes = html! {
            table border=(0) {
                tr {
                    td align=("center") {
                        a href=(url()) { "View order <script>" }
                    }
                }
            }
        };

        assert_eq!(
            render(&nodes),
            concat!(
                r#"<table border="0"><tr><td align="center">"#,
                r#"<a href="https://example.com/order">View order &lt;script&gt;</a>"#,
                "</td></tr></table>",
            )
        );
    }

    #[test]
    fn void_and_empty_elements_use_a_self_closing_form() {
        let nodes = html! {
            div {
                img src=(url()) alt=("Logo");
                span { }
                br;
            }
        };

        assert_eq!(
            render(&nodes),
            concat!(
                r#"<div><img src="https://example.com/order" alt="Logo" />"#,
                "<span></span><br /></div>",
            )
        );
    }

    #[test]
    fn interpolates_an_expression_as_escaped_text() {
        let name = String::from("Ada & <Lovelace>");

        let nodes = html! {
            p { "Hello, " (name) "!" }
        };

        assert_eq!(render(&nodes), "<p>Hello, Ada &amp; &lt;Lovelace&gt;!</p>");
    }

    #[test]
    fn class_takes_a_validated_class_name() {
        let nodes = html! {
            td class=(ClassName::new("stack").expect("valid")) { "x" }
        };

        assert_eq!(render(&nodes), r#"<td class="stack">x</td>"#);
    }

    #[test]
    fn a_component_splices_into_a_tree() {
        use crate::components::{Button, Container};
        use crate::markup::Color;

        // Container::build returns one node, Button::build returns several.
        let container = Container::new().padding_x(0).padding_y(0).build();
        let button = Button::new(url(), "View order")
            .background(Color::hex("#2563eb").expect("valid"))
            .build();

        let nodes = html! {
            div {
                @(container)
                p { "then" }
                @[button]
            }
        };

        let html = render(&nodes);
        assert!(html.starts_with("<div><table"), "{html}");
        assert!(html.contains("<p>then</p>"), "{html}");
        assert!(html.contains("<!--[if mso]>"), "{html}");
        assert!(html.ends_with("</div>"), "{html}");
    }

    #[test]
    fn an_element_splices_without_wrapping_it_first() {
        let el = Element::new(Tag::Strong).text("bold");
        let nodes = html! { p { "a " @(el) } };

        assert_eq!(render(&nodes), "<p>a <strong>bold</strong></p>");
    }

    #[test]
    fn placeholders_are_filled_by_render_with() {
        use crate::markup::VarName;
        use crate::render::{Bindings, render_with};

        let nodes = html! {
            p { "Hi, " {{ name }} ", order " {{ order_id }} }
        };

        let name = VarName::new("name").expect("valid");
        let order = VarName::new("order_id").expect("valid");

        let vars = Bindings::new()
            .set(name, "Ada & <Lovelace>")
            .set(order, "42");

        assert_eq!(
            render_with(&nodes, &vars).expect("bound"),
            "<p>Hi, Ada &amp; &lt;Lovelace&gt;, order 42</p>"
        );
    }

    #[test]
    fn an_unfilled_placeholder_is_visible_in_the_output() {
        let nodes = html! { p { "Hi, " {{ name }} } };
        assert_eq!(render(&nodes), "<p>Hi, {{name}}</p>");
    }

    /// The macro is syntax only. A hostile string reaching an attribute or a
    /// text node is escaped exactly as it is through the builder API, and
    /// `href` cannot be handed anything but a parsed `Url`.
    #[test]
    fn the_macro_opens_no_new_path() {
        let hostile = r#"" onerror="alert(1)"#;

        let nodes = html! {
            img src=(url()) alt=(hostile);
        };

        let html = render(&nodes);
        assert!(html.contains("&quot; onerror=&quot;alert(1)"), "{html}");
        assert!(!html.contains(r#"onerror="alert"#), "{html}");
    }
}
