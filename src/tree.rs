use super::parser::{Tree as UTree, TreeKind as Tk};
use std::fmt;

#[derive(Debug)]
pub enum Tree {
    Text(String),
    Inner {
        tag_name: String,
        attrs: Attrs,
        children: Vec<Tree>,
    },
}

pub type Attrs = Vec<(String, String)>;

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Tree::*;

        match self {
            Text(text) => write!(f, "{}", text),
            Inner {
                tag_name,
                attrs,
                children,
            } => {
                write!(f, "<{}", tag_name)?;
                if attrs.len() > 0 {
                    write!(f, " ")?;
                }
                for attr in attrs {
                    let (name, value) = attr;
                    write!(f, "{}=\"{}\"", name, value)?;
                }
                write!(f, ">")?;

                for child in children {
                    child.fmt(f)?;
                }

                write!(f, "</{}>", tag_name)?;

                Ok(())
            }
        }
    }
}

impl From<UTree> for Tree {
    fn from(tree: UTree) -> Self {
        match tree.kind {
            Tk::Document => parse_doc(tree),
            _ => panic!("expected document"),
        }
    }
}

fn parse_doc(mut tree: UTree) -> Tree {
    assert_eq!(tree.kind, Tk::Document);

    let inner = tree.children.pop().expect("expected child");

    if tree.children.len() > 0 {
        panic!("expected single child")
    }

    parse_inner(inner)
}

fn parse_inner(mut tree: UTree) -> Tree {
    assert_eq!(tree.kind, Tk::InnerNode);

    let end = tree.children.len() - 1;
    tree.children.swap(0, end);

    let open_tag = tree.children.pop().expect("expected open tag");
    let open_tag = parse_open_tag(open_tag);

    let close_tag = tree.children.get(0).expect("expected close tag");
    assert_eq!(close_tag.kind, Tk::CloseTag);

    let children = tree.children.into_iter().skip(1).map(parse_node).collect();

    Tree::Inner {
        tag_name: open_tag.name,
        attrs: open_tag.attrs,
        children,
    }
}

fn parse_node(tree: UTree) -> Tree {
    match tree.kind {
        Tk::InnerNode => parse_inner(tree),
        Tk::TextNode(content) => Tree::Text(content),
        _ => panic!("expected inner node or text"),
    }
}

fn parse_open_tag(mut tree: UTree) -> OpenTag {
    assert_eq!(tree.kind, Tk::OpenTag);

    let attrs = tree.children.pop().expect("expected attributes");
    let tag_name = tree.children.pop().expect("expected tag name");

    let attrs = parse_attrs(attrs);

    match tag_name.kind {
        Tk::TagName(name) => OpenTag { name, attrs },
        _ => panic!("expected tag name kind"),
    }
}

fn parse_attrs(tree: UTree) -> Attrs {
    assert_eq!(tree.kind, Tk::Attrs);

    tree.children.into_iter().map(parse_attr).collect()
}

fn parse_attr(mut tree: UTree) -> (String, String) {
    assert_eq!(tree.kind, Tk::Attr);

    let value = tree.children.pop().expect("expected attribute value");
    let name = tree.children.pop().expect("expected attribute name");

    match (name.kind, value.kind) {
        (Tk::AttrName(name), Tk::AttrVal(value)) => (name, value),
        (Tk::AttrName(_), _) => panic!("expected attribute value kind"),
        (_, Tk::AttrVal(_)) => panic!("expected attribute name kind"),
        _ => panic!("expected attribute name and attribute value kinds"),
    }
}

struct OpenTag {
    name: String,
    attrs: Attrs,
}
