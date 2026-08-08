pub struct RawHtml(String);

pub enum Node {
    Element(Element),
    Text(String),
    Raw(RawHtml),
    Conditional {
        cond: String,
        children: Vec<Node>
    },
}

pub struct Element {
    tag: Tag,
    attrs: Vec<(AttrName, AttrValue)>,
    class: Vec<ClassName>,
    styles: StyleMap,
    children: Vec<Node>,
}
