use crate::common::Span;
use std::fmt;

#[derive(Debug)]
pub struct SyntaxError<'a> {
    pub source: &'a str,
    pub span: Span,
    pub message: String,
}

impl<'a> SyntaxError<'a> {
    pub fn new<S>(source: &'a str, span: Span, message: S) -> Self
    where
        S: Into<String>,
    {
        SyntaxError {
            source,
            span,
            message: message.into(),
        }
    }
}

impl<'a> fmt::Display for SyntaxError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Span { start, end } = self.span;
        let start_line = pos_to_line(start, self.source);
        let end_line = pos_to_line(end, self.source);
        let lines = self
            .source
            .lines()
            .skip(start_line - 1)
            .take(end_line + 1 - start_line);

        for line in lines {
            writeln!(f, "{}", line)?;
            write!(f, " ^^^^^")?;
        }

        Ok(())
    }
}

fn pos_to_line(mut pos: usize, source: &str) -> usize {
    let mut line = 1;
    let mut chars = source.chars();

    while let Some(c) = chars.next() {
        if pos == 0 {
            break;
        }

        pos -= 1;
        match c {
            '\n' => line += 1,
            '\r' => match chars.next() {
                // We've just seen a CRLF
                Some('\n') => {
                    line += 1;
                }
                // We've just seen TWO CRs
                Some('\r') => {
                    line += 2;
                }
                _ => {}
            },
            _ => {}
        }
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_line_simple() {
        let src = "first\nsecond\r\nthird";
        //         012345 6789012 3 45678

        assert_eq!(pos_to_line(3, src), 1);
        assert_eq!(pos_to_line(6, src), 2);
        assert_eq!(pos_to_line(15, src), 3);
        assert_eq!(pos_to_line(451, src), 3);
    }

    #[test]
    fn get_line_double_cr() {
        let src = "first\nsecond\r\rthird";
        //         012345 6789012 3 45678

        assert_eq!(pos_to_line(15, src), 4);
    }
}
