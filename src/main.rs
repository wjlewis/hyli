use hyli::{Lexer, TokenKind};

fn main() {
    let input = r#"
<Doc title="Test doc">
    <Title>My first Doc</Title>

    <Section ref="sec-1">This is section 1</Section>

    Some text with a <Link to="sec-1">link</Link> to section 1.

    <CodeListing.Haskell #>
        reverse' :: [a] -> Latent [a] [a]
        reverse' = reset . aux
          where
            aux [] = []
            aux (x:xs) = shift $ \k -> return (x : k (aux xs))
    </# CodeListing.Haskell>

    <CodeListing.Hyli ##>
        <Doc title="This is a title">
            <CodeListing.Racket #>
                (define (map f xs)
                  (if (empty? xs)
                      '()
                      (cons (f (first xs))
                            (map f (rest xs)))))
            </# CodeListing.Racket>
        </Doc>
    </## CodeListing.Hyli>
</Doc>"#;

    let mut lexer = Lexer::from(input);

    while lexer.peek().kind != TokenKind::Eof {
        let token = lexer.pop();

        let text = &input[token.start..token.end];
        println!("{:?}\t`{}`", token.kind, text);
    }
}
