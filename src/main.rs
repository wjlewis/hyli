use hyli::parse;

fn main() {
    let input = r#"
<Doc title="My first doc">
    <Mixed><Up></Mixed></Up>
</Doc>
"#;

    println!("{:?}", parse(input));
}
