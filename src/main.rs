use hyli::{parse, Attrs, Processor, Tree};

fn main() {
    let mut p = Processor::new();
    p.add_transform("Doc", transform_doc);
    p.add_transform("Title", transform_title);

    let input = r#"
<Doc title="My first doc">
    <Title>My title</Title>

    <Mixed><Up></Mixed></Up>

    <Section ref="sec-1">First Section</Section>
</Doc>
"#;

    let result = parse(input);
    if result.errors.len() == 0 {
        let transformed = p.process(Tree::from(result.tree));
        println!("{}", transformed);
    } else {
        result.errors.iter().for_each(|error| {
            println!("{}", error.message);
        });
    }
}

fn transform_doc(_attrs: Attrs, children: Vec<Tree>) -> Tree {
    Tree::Inner {
        tag_name: String::from("html"),
        attrs: vec![(String::from("lang"), String::from("en-us"))],
        children: vec![
            Tree::Inner {
                tag_name: String::from("head"),
                attrs: vec![],
                children: vec![Tree::Text(String::from(" "))],
            },
            Tree::Inner {
                tag_name: String::from("body"),
                attrs: vec![],
                children,
            },
        ],
    }
}

fn transform_title(_attrs: Attrs, children: Vec<Tree>) -> Tree {
    Tree::Inner {
        tag_name: String::from("h1"),
        attrs: vec![(String::from("class"), String::from("title"))],
        children,
    }
}
