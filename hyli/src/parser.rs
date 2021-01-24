use crate::lexer::{Lexer, TokenKind};
use std::collections::HashMap;

use TokenKind::*;

#[derive(Debug)]
pub enum Tree {
  Inner {
    tag_name: String,
    attrs: Attrs,
    children: Vec<Tree>,
  },
  Text(String),
}

#[derive(Debug)]
pub struct SyntaxError {
  message: String,
}

#[derive(Debug)]
pub struct ParseResult<A> {
  result: Option<A>,
  errors: Vec<SyntaxError>,
}

type Attrs = HashMap<String, String>;

struct Attr {
  name: String,
  value: String,
}

pub fn parse<'a>(input: &'a str) -> ParseResult<Tree> {
  parse_tree(&mut DocItems::new(input))
}

fn parse_tree(items: &mut DocItems) -> ParseResult<Tree> {
  match items.next() {
    DocItem::OpenTag { tag_name, attrs } => {
      let children = parse_trees(items, &tag_name);

      ParseResult {
        result: Some(Tree::Inner {
          tag_name,
          attrs,
          children: children.result.unwrap_or(vec![]),
        }),
        errors: children.errors,
      }
    }
    DocItem::Text(text) => ParseResult {
      result: Some(Tree::Text(text)),
      errors: vec![],
    },
    _ => ParseResult {
      result: None,
      errors: vec![SyntaxError {
        message: "Expected an open tag or text".to_string(),
      }],
    },
  }
}

fn parse_trees(items: &mut DocItems, stop_tag: &str) -> ParseResult<Vec<Tree>> {
  let mut trees = vec![];
  let mut errors = vec![];

  loop {
    match items.peek() {
      DocItem::CloseTag { tag_name } => {
        if tag_name == stop_tag {
          items.next();
          return ParseResult {
            result: Some(trees),
            errors,
          };
        } else {
          errors.push(SyntaxError {
            message: format!(
              "Mismatched closing tags: expected </{}> but found </{}>",
              stop_tag, tag_name
            ),
          });
          items.next();
          return ParseResult {
            result: Some(trees),
            errors,
          };
        }
      }
      DocItem::Eof => {
        errors.push(SyntaxError {
          message: format!("Unexpected EOF while looking for </{}>", stop_tag),
        });
        return ParseResult {
          result: Some(trees),
          errors,
        };
      }
      _ => {
        let mut res = parse_tree(items);
        errors.append(&mut res.errors);

        if let Some(tree) = res.result {
          trees.push(tree);
        }
      }
    }
  }
}

#[derive(Debug, PartialEq)]
enum DocItem {
  OpenTag { tag_name: String, attrs: Attrs },
  CloseTag { tag_name: String },
  Text(String),
  Eof,
}

struct DocItems<'a> {
  lexer: Lexer<'a>,
  buffer: Option<DocItem>,
  errors: Vec<SyntaxError>,
}

impl<'a> DocItems<'a> {
  pub fn new(input: &'a str) -> Self {
    DocItems {
      lexer: Lexer::new(input),
      buffer: None,
      errors: Vec::new(),
    }
  }

  pub fn peek(&mut self) -> &DocItem {
    if self.buffer.is_none() {
      self.buffer = Some(self.read_next());
    }

    self.buffer.as_ref().unwrap()
  }

  pub fn next(&mut self) -> DocItem {
    if self.buffer.is_some() {
      self.buffer.take().unwrap()
    } else {
      self.read_next()
    }
  }

  fn read_next(&mut self) -> DocItem {
    match self.lexer.peek().kind {
      Eof => DocItem::Eof,
      LAngle => {
        let mut res = self.parse_open_tag();
        self.errors.append(&mut res.errors);
        if let Some(result) = res.result {
          result
        } else {
          self.read_next()
        }
      }
      LAngleSlash => {
        let mut res = self.parse_close_tag();
        self.errors.append(&mut res.errors);
        if let Some(result) = res.result {
          result
        } else {
          self.read_next()
        }
      }
      Text => {
        let text = self.lexer.next().text;
        DocItem::Text(text)
      }
      _ => {
        self.errors.push(SyntaxError {
          message: "Expected <, >, or text".to_string(),
        });
        self.lexer.next();
        self.read_next()
      }
    }
  }

  fn parse_open_tag(&mut self) -> ParseResult<DocItem> {
    self.lexer.next();

    if self.lexer.peek().kind == Name {
      let tag_name = self.lexer.next().text;

      let mut attrs_parse = self.parse_attrs();

      if let Some(attrs) = attrs_parse.result {
        if self.lexer.peek().kind == RAngle {
          self.lexer.next();

          ParseResult {
            result: Some(DocItem::OpenTag { tag_name, attrs }),
            errors: attrs_parse.errors,
          }
        } else {
          let mut errors = vec![SyntaxError {
            message: "Expected > to close tag".to_string(),
          }];
          errors.append(&mut attrs_parse.errors);
          ParseResult {
            result: None,
            errors,
          }
        }
      } else {
        ParseResult {
          result: Some(DocItem::OpenTag {
            tag_name,
            attrs: HashMap::new(),
          }),
          errors: attrs_parse.errors,
        }
      }
    } else {
      ParseResult {
        result: None,
        errors: vec![SyntaxError {
          message: "Expected tag name".to_string(),
        }],
      }
    }
  }

  fn parse_close_tag(&mut self) -> ParseResult<DocItem> {
    self.lexer.next();

    if self.lexer.peek().kind == Name {
      let tag_name = self.lexer.next().text;

      if self.lexer.peek().kind == RAngle {
        self.lexer.next();

        ParseResult {
          result: Some(DocItem::CloseTag { tag_name }),
          errors: vec![],
        }
      } else {
        ParseResult {
          result: None,
          errors: vec![SyntaxError {
            message: "Expected > to close tag".to_string(),
          }],
        }
      }
    } else {
      ParseResult {
        result: None,
        errors: vec![SyntaxError {
          message: "Expected tag name".to_string(),
        }],
      }
    }
  }

  fn parse_attrs(&mut self) -> ParseResult<Attrs> {
    let mut attrs = HashMap::new();
    let mut errors = Vec::new();

    while self.lexer.peek().kind == Name {
      let mut parse = self.parse_attr();
      errors.append(&mut parse.errors);
      if let Some(attr) = parse.result {
        attrs.insert(attr.name, attr.value);
      }
    }

    ParseResult {
      result: Some(attrs),
      errors,
    }
  }

  fn parse_attr(&mut self) -> ParseResult<Attr> {
    let name = self.lexer.next().text;

    if self.lexer.peek().kind == Equals {
      self.lexer.next();

      if self.lexer.peek().kind == AttrVal {
        let value = self.lexer.next().text;

        ParseResult {
          result: Some(Attr { name, value }),
          errors: vec![],
        }
      } else {
        ParseResult {
          result: None,
          errors: vec![SyntaxError {
            message: "Expected an attribute value".to_string(),
          }],
        }
      }
    } else {
      ParseResult {
        result: None,
        errors: vec![SyntaxError {
          message: "Expected an equals sign".to_string(),
        }],
      }
    }
  }
}
