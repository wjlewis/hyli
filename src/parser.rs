use super::common::Span;
use super::lexer::{Lexer, TokenKind as Tk};
use std::fmt;

#[derive(PartialEq)]
pub struct Tree {
    pub kind: TreeKind,
    pub span: Span,
    pub children: Vec<Tree>,
}

impl fmt::Debug for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_debug(f, 0)
    }
}

impl Tree {
    fn fmt_debug(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        let indent = " ".repeat(depth * 2);
        writeln!(f, "{}{:?}@{:?}", indent, self.kind, self.span)?;

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
    builder.open(TreeKind::Document, tokens.peek().start());
    loop {
        let peek = tokens.peek();
        match peek.kind {
            Tk::LAngle => break,
            Tk::Eof => {
                builder.add_error(SyntaxError::new(peek.span, "unexpected EOF"));
                builder.complete(peek.start());
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

    builder.complete(tokens.peek().start());
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
                    peek.span,
                    format!(r#"expected '<', "</", or text, but found "{}""#, peek.text),
                ));
                tokens.pop();
            }
        }
    }
}

fn parse_inner_node(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    builder.open(TreeKind::InnerNode, tokens.peek().start());
    let open_tag_name = parse_open_tag(builder, tokens);
    parse_nodes(builder, tokens);

    let peek = tokens.peek();
    if peek.kind == Tk::Eof {
        builder.complete(peek.start());
        builder.add_error(SyntaxError::new(
            peek.span,
            "expected closing tag, but found EOF",
        ));
        return;
    }

    let close_tag = parse_close_tag(builder, tokens);
    builder.complete(tokens.peek().start());

    match (open_tag_name, close_tag) {
        (Some(open), Some(CloseTag { name, span })) if open != name => {
            builder.add_error(SyntaxError::new(
                span,
                format!(
                    r#"closing tag must match opening (expected "{}" but found "{}")"#,
                    open, name
                ),
            ))
        }
        _ => {}
    }
}

fn parse_text_node(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    let text = tokens.pop();
    builder.add_leaf(TreeKind::TextNode(text.text), text.span);
}

fn parse_open_tag(builder: &mut TreeBuilder, tokens: &mut Lexer) -> Option<String> {
    let mut tag_name = None;
    let langle = tokens.pop();
    builder.open(TreeKind::OpenTag, langle.start());

    let peek = tokens.peek();
    match peek.kind {
        Tk::Name => {
            let name = tokens.pop();
            tag_name = Some(name.text.clone());
            builder.add_leaf(TreeKind::TagName(name.text), name.span);
        }
        Tk::Equals | Tk::AttrVal | Tk::RAngle => {
            builder.add_error(SyntaxError::new(peek.span, "expected tag name"));
        }
        _ => {
            builder.add_error(SyntaxError::new(
                peek.span,
                "expected tag name, followed by attributes and '>'",
            ));
            builder.complete(peek.start());
            return tag_name;
        }
    }

    parse_attrs(builder, tokens);

    let peek = tokens.peek();
    let end = peek.end();
    match peek.kind {
        Tk::RAngle => {
            tokens.pop();
        }
        _ => {
            builder.add_error(SyntaxError::new(peek.span, "expected '>'"));
            builder.complete(peek.start());
            return tag_name;
        }
    }

    builder.complete(end);
    tag_name
}

fn parse_close_tag(builder: &mut TreeBuilder, tokens: &mut Lexer) -> Option<CloseTag> {
    let mut tag_info = None;
    let langle_slash = tokens.pop();
    builder.open(TreeKind::CloseTag, langle_slash.start());

    if tokens.peek().kind == Tk::OrphanHashes {
        let orphans = tokens.pop();
        builder.add_error(SyntaxError::new(orphans.span, "orphaned hashes"));
    }

    let peek = tokens.peek();
    match peek.kind {
        Tk::Name => {
            let name = tokens.pop();
            tag_info = Some(CloseTag {
                name: name.text.clone(),
                span: name.span,
            });
            builder.add_leaf(TreeKind::TagName(name.text), name.span);
        }
        Tk::RAngle => {
            builder.add_error(SyntaxError::new(peek.span, "expected tag name"));
        }
        _ => {
            builder.add_error(SyntaxError::new(
                peek.span,
                "expected tag name, followed by '>'",
            ));
            builder.complete(peek.start());
            return tag_info;
        }
    }

    let peek = tokens.peek();
    let end = peek.end();
    match peek.kind {
        Tk::RAngle => {
            tokens.pop();
        }
        _ => {
            builder.add_error(SyntaxError::new(peek.span, "expected '>'"));
            builder.complete(peek.start());
            return tag_info;
        }
    }

    builder.complete(end);
    tag_info
}

fn parse_attrs(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    builder.open(TreeKind::Attrs, tokens.peek().start());

    while tokens.peek().kind == Tk::Name {
        parse_attr(builder, tokens);
    }

    builder.complete(tokens.peek().start());
}

fn parse_attr(builder: &mut TreeBuilder, tokens: &mut Lexer) {
    let name = tokens.pop();
    builder.open(TreeKind::Attr, name.start());
    builder.add_leaf(TreeKind::AttrName(name.text), name.span);

    let peek = tokens.peek();
    match peek.kind {
        Tk::Equals => {
            tokens.pop();
        }
        Tk::AttrVal | Tk::UnterminatedAttrVal => {
            builder.add_error(SyntaxError::new(peek.span, "expected '='"));
        }
        _ => {
            builder.add_error(SyntaxError::new(
                peek.span,
                "expected '=', followed by attribute value",
            ));
            builder.complete(peek.start());
            return;
        }
    }

    let peek = tokens.peek();
    let end = peek.end();
    match peek.kind {
        Tk::AttrVal | Tk::UnterminatedAttrVal => {
            if peek.kind == Tk::UnterminatedAttrVal {
                builder.add_error(SyntaxError::new(peek.span, "unterminated attribute value"));
            }

            let attr_val = tokens.pop();
            builder.add_leaf(TreeKind::AttrVal(attr_val.text), attr_val.span);
        }
        _ => {
            builder.add_error(SyntaxError::new(peek.span, "expected attribute value"));
            builder.complete(peek.start());
            return;
        }
    }

    builder.complete(end);
}

struct CloseTag {
    name: String,
    span: Span,
}

#[derive(Debug)]
pub struct ParseResult {
    pub tree: Tree,
    pub errors: Vec<SyntaxError>,
}

#[derive(Debug)]
pub struct SyntaxError {
    pub span: Span,
    pub message: String,
}

impl SyntaxError {
    fn new<S>(span: Span, message: S) -> Self
    where
        S: Into<String>,
    {
        SyntaxError {
            span,
            message: message.into(),
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
        use BuilderItem::*;
        let item = self.wip.pop().expect("no tree to take");

        let tree = match item {
            Complete(tree) => tree,
            _ => panic!(""),
        };

        if self.wip.len() > 0 {
            panic!("multiple trees in WIP");
        }

        ParseResult {
            tree,
            errors: self.errors,
        }
    }

    fn open(&mut self, kind: TreeKind, start: usize) {
        self.wip.push(BuilderItem::InProgress { kind, start });
    }

    fn add_leaf(&mut self, kind: TreeKind, span: Span) {
        self.wip.push(BuilderItem::Complete(Tree {
            kind,
            span,
            children: vec![],
        }));
    }

    fn complete(&mut self, end: usize) {
        let mut children = vec![];

        while let Some(item) = self.wip.pop() {
            match item {
                BuilderItem::InProgress { kind, start } => {
                    children.reverse();
                    self.wip.push(BuilderItem::Complete(Tree {
                        kind,
                        span: Span::new(start, end),
                        children,
                    }));
                    return;
                }
                BuilderItem::Complete(tree) => {
                    children.push(tree);
                }
            }
        }

        panic!("no open item to complete");
    }

    fn add_error(&mut self, error: SyntaxError) {
        self.errors.push(error);
    }
}

enum BuilderItem {
    InProgress { kind: TreeKind, start: usize },
    Complete(Tree),
}
