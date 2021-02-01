use super::lexer::{Lexer, TokenKind as Tk};
use std::fmt;

#[derive(PartialEq)]
pub struct Tree {
    kind: TreeKind,
    start: usize,
    end: usize,
    children: Vec<Tree>,
}

impl fmt::Debug for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_debug(f, 0)
    }
}

impl Tree {
    fn fmt_debug(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        let indent = " ".repeat(depth * 2);
        writeln!(f, "{}{:?}@{}..{}", indent, self.kind, self.start, self.end)?;

        for child in &self.children {
            child.fmt_debug(f, depth + 1)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum TreeKind {
    Document,
    InnerNode,
    OpenTag,
    TagName(String),
    Attrs,
    Attr,
    AttrName(String),
    AttrVal(String),
    CloseTag,
    TextNode(String),
}

pub fn parse<'a>(input: &'a str) -> ParseResult {
    let mut tokens = Lexer::from(input);
    let mut builder = TreeBuilder::new();

    parse_document(&mut builder, &mut tokens);

    builder.take()
}

fn parse_document(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    builder.open(TreeKind::Document, tokens.peek().start);
    loop {
        let peek = tokens.peek();
        match peek.kind {
            Tk::LAngle => break,
            Tk::Eof => {
                builder.add_error(SyntaxError::new("unexpected EOF", peek.start, peek.end));
                builder.complete(peek.start);
                return;
            }
            _ => {
                tokens.pop();
            }
        }
    }

    parse_inner_node(builder, tokens);

    loop {
        let peek = tokens.peek();
        match peek.kind {
            Tk::Eof => break,
            _ => {
                tokens.pop();
            }
        }
    }

    builder.complete(tokens.peek().start);
}

fn parse_nodes(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    loop {
        let peek = tokens.peek();
        match peek.kind {
            Tk::LAngleSlash | Tk::Eof => return,
            Tk::LAngle => parse_inner_node(builder, tokens),
            Tk::Text => parse_text_node(builder, tokens),
            _ => {
                builder.add_error(SyntaxError::new(
                    format!(r#"expected '<', "</", or text, but found "{}""#, peek.text),
                    peek.start,
                    peek.end,
                ));
                tokens.pop();
            }
        }
    }
}

fn parse_inner_node(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    builder.open(TreeKind::InnerNode, tokens.peek().start);
    let open_tag_name = parse_open_tag(builder, tokens);
    parse_nodes(builder, tokens);

    let peek = tokens.peek();
    if peek.kind == Tk::Eof {
        builder.complete(peek.start);
        builder.add_error(SyntaxError::new(
            "expected closing tag, but found EOF",
            peek.start,
            peek.end,
        ));
        return;
    }

    let close_tag = parse_close_tag(builder, tokens);
    builder.complete(tokens.peek().start);

    match (open_tag_name, close_tag) {
        (Some(open), Some(CloseTag { name, start, end })) if open != name => {
            builder.add_error(SyntaxError::new(
                format!(
                    r#"closing tag must match opening (expected "{}" but found "{}")"#,
                    open, name
                ),
                start,
                end,
            ))
        }
        _ => {}
    }
}

fn parse_text_node(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    let text = tokens.pop();
    builder.open(TreeKind::TextNode(text.text), text.start);
    builder.complete(text.end);
}

fn parse_open_tag(builder: &mut TreeBuilder, tokens: &mut Lexer) -> Option<String> {
    let mut tag_name = None;
    let langle = tokens.pop();
    builder.open(TreeKind::OpenTag, langle.start);

    let peek = tokens.peek();
    match peek.kind {
        Tk::Name => {
            let name = tokens.pop();
            tag_name = Some(name.text.clone());
            builder.open(TreeKind::TagName(name.text), name.start);
            builder.complete(name.end);
        }
        Tk::Equals | Tk::AttrVal | Tk::RAngle => {
            builder.add_error(SyntaxError::new("expected tag name", peek.start, peek.end));
        }
        _ => {
            builder.add_error(SyntaxError::new(
                "expected tag name, followed by attributes and '>'",
                peek.start,
                peek.end,
            ));
            builder.complete(peek.start);
            return tag_name;
        }
    }

    parse_attrs(builder, tokens);

    let peek = tokens.peek();
    let end = peek.end;
    match peek.kind {
        Tk::RAngle => {
            tokens.pop();
        }
        _ => {
            builder.add_error(SyntaxError::new("expected '>'", peek.start, peek.end));
            builder.complete(peek.start);
            return tag_name;
        }
    }

    builder.complete(end);
    tag_name
}

fn parse_close_tag(builder: &mut TreeBuilder, tokens: &mut Lexer) -> Option<CloseTag> {
    let mut tag_info = None;
    let langle_slash = tokens.pop();
    builder.open(TreeKind::CloseTag, langle_slash.start);

    if tokens.peek().kind == Tk::OrphanHashes {
        let orphans = tokens.pop();
        builder.add_error(SyntaxError::new(
            "orphaned hashes",
            orphans.start,
            orphans.end,
        ));
    }

    let peek = tokens.peek();
    match peek.kind {
        Tk::Name => {
            let name = tokens.pop();
            tag_info = Some(CloseTag {
                name: name.text.clone(),
                start: name.start,
                end: name.end,
            });
            builder.open(TreeKind::TagName(name.text), name.start);
            builder.complete(name.end);
        }
        Tk::RAngle => {
            builder.add_error(SyntaxError::new("expected tag name", peek.start, peek.end));
        }
        _ => {
            builder.add_error(SyntaxError::new(
                "expected tag name, followed by '>'",
                peek.start,
                peek.end,
            ));
            builder.complete(peek.start);
            return tag_info;
        }
    }

    let peek = tokens.peek();
    let end = peek.end;
    match peek.kind {
        Tk::RAngle => {
            tokens.pop();
        }
        _ => {
            builder.add_error(SyntaxError::new("expected '>'", peek.start, peek.end));
            builder.complete(peek.start);
            return tag_info;
        }
    }

    builder.complete(end);
    tag_info
}

fn parse_attrs(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    builder.open(TreeKind::Attrs, tokens.peek().start);

    while tokens.peek().kind == Tk::Name {
        parse_attr(builder, tokens);
    }

    builder.complete(tokens.peek().start);
}

fn parse_attr(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    let name = tokens.pop();
    builder.open(TreeKind::Attr, name.start);
    builder.open(TreeKind::AttrName(name.text), name.start);
    builder.complete(name.end);

    let peek = tokens.peek();
    match peek.kind {
        Tk::Equals => {
            tokens.pop();
        }
        Tk::AttrVal | Tk::UnterminatedAttrVal => {
            builder.add_error(SyntaxError::new("expected '='", peek.start, peek.end));
        }
        _ => {
            builder.add_error(SyntaxError::new(
                "expected '=', followed by attribute value",
                peek.start,
                peek.end,
            ));
            builder.complete(peek.start);
            return;
        }
    }

    let peek = tokens.peek();
    let end = peek.end;
    match peek.kind {
        Tk::AttrVal | Tk::UnterminatedAttrVal => {
            if peek.kind == Tk::UnterminatedAttrVal {
                builder.add_error(SyntaxError::new(
                    "unterminated attribute value",
                    peek.start,
                    peek.end,
                ));
            }

            let attr_val = tokens.pop();
            builder.open(TreeKind::AttrVal(attr_val.text), attr_val.start);
            builder.complete(attr_val.end);
        }
        _ => {
            builder.add_error(SyntaxError::new(
                "expected attribute value",
                peek.start,
                peek.end,
            ));
            builder.complete(peek.start);
            return;
        }
    }

    builder.complete(end);
}

struct CloseTag {
    name: String,
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub struct ParseResult {
    result: Tree,
    errors: Vec<SyntaxError>,
}

#[derive(Debug)]
pub struct SyntaxError {
    message: String,
    start: usize,
    end: usize,
}

impl SyntaxError {
    fn new<S>(message: S, start: usize, end: usize) -> Self
    where
        S: Into<String>,
    {
        SyntaxError {
            message: message.into(),
            start,
            end,
        }
    }
}

struct TreeBuilder {
    wip: Vec<BuilderItem>,
    errors: Vec<SyntaxError>,
}

impl TreeBuilder {
    fn new() -> Self {
        TreeBuilder {
            wip: vec![],
            errors: vec![],
        }
    }

    fn take(mut self) -> ParseResult {
        if self.wip.len() == 0 {
            panic!("No tree");
        } else if self.wip.len() > 1 {
            panic!("Unmatched `open`");
        } else if let Some(BuilderItem::Complete { tree }) = self.wip.pop() {
            ParseResult {
                result: tree,
                errors: self.errors,
            }
        } else {
            panic!("Unmatched `open`");
        }
    }

    fn open(&mut self, kind: TreeKind, start: usize) {
        self.wip.push(BuilderItem::InProgress { kind, start });
    }

    fn insert(&mut self, tree: Tree) {
        self.wip.push(BuilderItem::Complete { tree });
    }

    fn complete(&mut self, end: usize) {
        let mut children = vec![];

        while let Some(item) = self.wip.pop() {
            match item {
                BuilderItem::InProgress { kind, start } => {
                    children.reverse();
                    self.wip.push(BuilderItem::Complete {
                        tree: Tree {
                            kind,
                            start,
                            end,
                            children,
                        },
                    });
                    return;
                }
                BuilderItem::Complete { tree } => {
                    children.push(tree);
                }
            }
        }

        panic!("No open item to complete");
    }

    fn add_error(&mut self, error: SyntaxError) {
        self.errors.push(error);
    }
}

enum BuilderItem {
    InProgress { kind: TreeKind, start: usize },
    Complete { tree: Tree },
}
