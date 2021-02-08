use super::common::Span;
use std::str::Chars;

pub struct Lexer<'a> {
    pub input: &'a str,
    chars: Chars<'a>,
    mode: LexerMode,
    buffer: Option<Token>,
}

impl<'a> Lexer<'a> {
    pub fn peek(&mut self) -> &Token {
        if self.buffer.is_none() {
            self.buffer = Some(self.read_next());
        }

        self.buffer.as_ref().unwrap()
    }

    pub fn pop(&mut self) -> Token {
        match self.buffer {
            Some(_) => self.buffer.take().unwrap(),
            None => self.read_next(),
        }
    }

    fn read_next(&mut self) -> Token {
        match self.mode {
            LexerMode::Inside(hash_count) => self.read_inside(hash_count),
            LexerMode::Outside(hash_count) => self.read_outside(hash_count),
        }
    }

    fn read_inside(&mut self, hash_count: usize) -> Token {
        self.skip_whitespace();

        let mut start = self.current_pos();
        if self.peek_char().is_none() {
            return Token::eof(start);
        }

        let kind = match self.chars.next().unwrap() {
            '<' => self.read_langle(hash_count),
            '>' => self.read_rangle(0),
            '#' => self.read_hashes(),
            '=' => TokenKind::Equals,
            '"' => self.read_attr_val(),
            c if is_name_start(c) => self.read_name(),
            _ => {
                self.mode = LexerMode::Outside(0);
                return self.read_next();
            }
        };

        let mut end = self.current_pos();

        // Adjust start and end positions for quoted values (to exclude
        // quotes).
        if kind == TokenKind::AttrVal {
            start += 1;
            end -= 1;
        } else if kind == TokenKind::UnterminatedAttrVal {
            start += 1;
        }

        Token::from_input(kind, self.input, start, end)
    }

    fn read_langle(&mut self, hash_count: usize) -> TokenKind {
        if let Some('/') = self.peek_char() {
            self.chars.next();
            if hash_count > 0 {
                self.chars.nth(hash_count - 1);
            }

            self.mode = LexerMode::Inside(0);
            TokenKind::LAngleSlash
        } else {
            TokenKind::LAngle
        }
    }

    fn read_rangle(&mut self, hash_count: usize) -> TokenKind {
        self.mode = LexerMode::Outside(hash_count);
        TokenKind::RAngle
    }

    fn read_hashes(&mut self) -> TokenKind {
        let hash_count = 1 + self.skip_while(|c| c == '#');

        if let Some('>') = self.peek_char() {
            self.chars.next();
            self.read_rangle(hash_count)
        } else {
            TokenKind::OrphanHashes
        }
    }

    fn read_attr_val(&mut self) -> TokenKind {
        let mut escape_next = false;

        while let Some(c) = self.peek_char() {
            if c == '\n' || c == '\r' {
                return TokenKind::UnterminatedAttrVal;
            }

            match self.chars.next().unwrap() {
                '\\' if !escape_next => {
                    escape_next = true;
                }
                '"' if !escape_next => {
                    return TokenKind::AttrVal;
                }
                _ => {
                    escape_next = false;
                }
            }
        }

        TokenKind::UnterminatedAttrVal
    }

    fn read_name(&mut self) -> TokenKind {
        self.skip_while(is_name_continue);
        TokenKind::Name
    }

    fn read_outside(&mut self, hash_count: usize) -> Token {
        let start = self.current_pos();

        let end = loop {
            match self.peek_char() {
                Some('<') => {
                    if hash_count == 0 {
                        break self.current_pos();
                    }

                    if let Some('/') = self.peek_nth(1) {
                        if self
                            .chars
                            .clone()
                            .skip(2)
                            .take(hash_count)
                            .all(|c| c == '#')
                        {
                            break self.current_pos();
                        }
                    }

                    self.chars.next();
                }
                Some(_) => {
                    self.chars.next();
                }
                None => break self.current_pos(),
            }
        };

        self.mode = LexerMode::Inside(hash_count);
        if end > start {
            Token::from_input(TokenKind::Text, self.input, start, end)
        } else {
            self.read_next()
        }
    }

    fn skip_whitespace(&mut self) {
        self.skip_while(is_whitespace);
    }

    fn skip_while<F>(&mut self, pred: F) -> usize
    where
        F: Fn(char) -> bool,
    {
        let mut count = 0;
        while let Some(c) = self.peek_char() {
            if !pred(c) {
                break;
            }

            count += 1;
            self.chars.next();
        }

        count
    }

    fn current_pos(&self) -> usize {
        self.input.len() - self.chars.as_str().len()
    }

    fn peek_char(&self) -> Option<char> {
        self.peek_nth(0)
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.chars.clone().nth(n)
    }
}

impl<'a> From<&'a str> for Lexer<'a> {
    fn from(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.chars(),
            mode: LexerMode::Outside(0),
            buffer: None,
        }
    }
}

fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\t' | '\n' | '\r' => true,
        _ => false,
    }
}

fn is_name_start(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' => true,
        _ => false,
    }
}

fn is_name_continue(c: char) -> bool {
    match c {
        c if is_name_start(c) => true,
        '0'..='9' | '.' | '_' | '-' => true,
        _ => false,
    }
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
}

impl Token {
    pub fn start(&self) -> usize {
        self.span.start
    }

    pub fn end(&self) -> usize {
        self.span.end
    }

    fn new<S>(kind: TokenKind, text: S, start: usize, end: usize) -> Self
    where
        S: Into<String>,
    {
        Token {
            kind,
            text: text.into(),
            span: Span::new(start, end),
        }
    }

    fn from_input(kind: TokenKind, input: &str, start: usize, end: usize) -> Self {
        Token::new(kind, &input[start..end], start, end)
    }

    fn eof(pos: usize) -> Self {
        Token::new(TokenKind::Eof, "", pos, pos)
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    LAngle,
    LAngleSlash,
    RAngle,
    Name,
    Equals,
    AttrVal,
    UnterminatedAttrVal,
    OrphanHashes,
    Text,
    Eof,
}

#[derive(PartialEq)]
pub enum LexerMode {
    Inside(usize),
    Outside(usize),
}
