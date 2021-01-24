use hyli::parse;

fn main() {
    let input = r#"
<Doc title="My first doc" date="1/24/2020">
    <Title>5 Elegant Uses for First-Class Continuations</Title>

    <Subtitle>Featuring <Mono>shift</Mono> and <Mono>reset</Mono></Subtitle>

    <CodeListing.Haskell desc="A preview of coming attractions">
reverse :: [a] -> Latent r [a]
reverse = reset . aux
  where
    aux :: [a] -> Latent [a] [a]
    aux [] = return []
    aux (x : xs) = shift $ \k -> x : k (aux xs)
    </CodeListing.Haskell>

    What have we here?
</Doc>
"#;

    println!("{:#?}", parse(input));
}
