use hyli::{run, Attrs, Processor, Tree};

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut proc = Processor::new();
    proc.add_transform("Doc", transform_doc);

    run("./test.xml", &proc)?;
    Ok(())
}

fn transform_doc(attrs: Attrs, children: Vec<Tree>) -> Tree {
    Tree::Inner {
        tag_name: String::from("html"),
        attrs: vec![],
        children: vec![],
    }
}
