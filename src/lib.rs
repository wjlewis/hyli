mod common;
mod file;
mod lexer;
mod parser;
mod processor;
mod syntax_error;
mod tree;

use common::FILE_INFO;
use file::read_file;

pub use processor::{Processor, Transform};
pub use tree::{Attrs, Tree};

pub fn run(path: &str, proc: &Processor) -> Result<(), Box<dyn std::error::Error + 'static>> {
    read_file(path)?;

    FILE_INFO.with(|info| {
        let info = info.borrow();
        let text = &info.text;
        let result = parser::parse(text);

        if result.errors.len() > 0 {
            eprintln!("ERRORS");
        } else {
            let out = proc.process(Tree::from(result.tree));
            println!("{}", out);
        }
    });

    Ok(())
}
