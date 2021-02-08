use hyli::parse;

fn main() {
    let input = r#"<Mixed><Up></Mixed></Up>"#;

    let result = parse(input);

    for error in result.errors {
        println!("{}", error);
    }
}
