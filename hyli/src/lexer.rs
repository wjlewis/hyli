use std::str::Chars;

pub struct Lexer<'a> {
  input: &'a str,
  chars: Chars<'a>,
  mode: LexerMode,
  buffer: Option<Token>,
}

impl<'a> Lexer<'a> {
  pub fn new(input: &'a str) -> Self {
    Lexer {
      input,
      chars: input.chars(),
      mode: LexerMode::OutsideTag(0),
      buffer: None,
    }
  }

  pub fn peek(&mut self) -> &Token {
    if self.buffer.is_none() {
      self.buffer = Some(self.read_next());
    }

    self.buffer.as_ref().unwrap()
  }

  pub fn next(&mut self) -> Token {
    if self.buffer.is_some() {
      self.buffer.take().unwrap()
    } else {
      self.read_next()
    }
  }

  fn read_next(&mut self) -> Token {
    self.skip_whitespace();
    let mut start = self.current_pos();
    let next = self.chars.next();

    if next.is_none() {
      return Token::eof();
    }

    let kind = match next.unwrap() {
      '<' => self.read_langle(),
      '>' => self.read_rangle(0),
      '#' => self.read_hashes(),
      '=' => TokenKind::Equals,
      '"' => {
        start += 1;
        self.read_attr_val()
      }
      _ if self.mode == LexerMode::InsideTag => self.read_name(),
      _ => self.read_text(),
    };

    let len = self.compute_len(start, kind);
    let text = &self.input[start..start + len];
    Token::new(kind, text.to_string(), start)
  }

  fn read_langle(&mut self) -> TokenKind {
    if self.is_rangle(0) {
      self.mode = LexerMode::InsideTag;

      if let Some('/') = self.peek_char() {
        self.chars.next();

        if let LexerMode::OutsideTag(hash_count) = self.mode {
          self.skip_while(|c, n| c == '#' && n <= hash_count);
        }

        TokenKind::LAngleSlash
      } else {
        TokenKind::LAngle
      }
    } else {
      self.read_text()
    }
  }

  fn read_rangle(&mut self, hash_count: usize) -> TokenKind {
    self.mode = LexerMode::OutsideTag(hash_count);
    TokenKind::RAngle
  }

  fn read_hashes(&mut self) -> TokenKind {
    // We've already read past the first hash; hence the `1 + ..`.
    let hash_count = 1 + self.skip_while(|c, _| c == '#');

    if let Some('>') = self.peek_char() {
      self.chars.next();
      self.read_rangle(hash_count)
    } else {
      TokenKind::OrphanHashes
    }
  }

  fn read_attr_val(&mut self) -> TokenKind {
    self.skip_while(|c, _| c != '"');
    if let Some('"') = self.peek_char() {
      self.chars.next();
      TokenKind::AttrVal
    } else {
      TokenKind::UnterminatedAttrVal
    }
  }

  fn read_name(&mut self) -> TokenKind {
    self.skip_while(|c, _| match c {
      '=' | '>' | '#' | ' ' | '\t' | '\n' => false,
      _ => true,
    });
    TokenKind::Name
  }

  fn read_text(&mut self) -> TokenKind {
    while let Some(c) = self.peek_char() {
      match c {
        '<' => {
          if self.is_rangle(1) {
            break;
          }
        }
        _ => {}
      }

      self.chars.next();
    }

    TokenKind::Text
  }

  fn is_rangle(&mut self, start: usize) -> bool {
    match self.mode {
      LexerMode::OutsideTag(n) => {
        if n == 0 {
          return true;
        }

        if let Some('/') = self.peek_nth(start) {
          let mut matches_hashes = true;

          for offset in start + 1..n + start + 1 {
            if let Some('#') = self.peek_nth(offset) {
              continue;
            } else {
              // Skip past the characters we've peeked, and continue
              // reading text.
              self.chars.nth(offset);
              matches_hashes = false;
              break;
            }
          }

          if matches_hashes {
            true
          } else {
            false
          }
        } else {
          false
        }
      }
      _ => true,
    }
  }

  fn compute_len(&self, start: usize, kind: TokenKind) -> usize {
    let current_pos = self.current_pos();

    match kind {
      TokenKind::Text => {
        let text = &self.input[start..current_pos];
        text.trim().len()
      }
      TokenKind::AttrVal => current_pos - 1 - start,
      _ => current_pos - start,
    }
  }

  fn current_pos(&self) -> usize {
    self.input.len() - self.chars.as_str().len()
  }

  fn skip_whitespace(&mut self) {
    self.skip_while(|c, _| match c {
      ' ' | '\n' | '\t' => true,
      _ => false,
    });
  }

  fn skip_while<F>(&mut self, pred: F) -> usize
  where
    F: Fn(char, usize) -> bool,
  {
    let mut count = 0;
    while let Some(c) = self.peek_char() {
      if !pred(c, count + 1) {
        break;
      }

      count += 1;
      self.chars.next();
    }

    count
  }

  fn peek_char(&mut self) -> Option<char> {
    self.peek_nth(0)
  }

  fn peek_nth(&mut self, n: usize) -> Option<char> {
    self.chars.clone().into_iter().nth(n)
  }
}

#[derive(Debug, PartialEq)]
enum LexerMode {
  InsideTag,
  OutsideTag(usize),
}

#[derive(Debug)]
pub struct Token {
  pub kind: TokenKind,
  pub text: String,
  pub start: usize,
}

impl Token {
  fn new(kind: TokenKind, text: String, start: usize) -> Self {
    Token { kind, text, start }
  }

  fn eof() -> Self {
    Token::new(TokenKind::Eof, "".to_string(), 0)
  }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenKind {
  LAngle,              // `<`
  LAngleSlash,         // `</#..`
  RAngle,              // `#..>`
  OrphanHashes,        // `#..`
  Name,                // `Doc` in `<Doc title="My Doc">`
  Equals,              // `=`
  AttrVal,             // `"My Doc"` in `<Doc title="My Doc">`
  UnterminatedAttrVal, // Invalid
  Text,                // `inner text` in `<span>inner text</span>`
  Eof,                 // EOF
}
