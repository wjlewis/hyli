use super::tree::{Attrs, Tree};
use std::collections::HashMap;

pub struct Processor {
    transforms: HashMap<String, Transform>,
}

pub type Transform = fn(Attrs, Vec<Tree>) -> Tree;

impl Processor {
    pub fn new() -> Self {
        Processor {
            transforms: HashMap::new(),
        }
    }

    pub fn add_transform<S>(&mut self, name: S, transform: Transform)
    where
        S: Into<String>,
    {
        self.transforms.insert(name.into(), transform);
    }

    pub fn process(&self, tree: Tree) -> Tree {
        match tree {
            Tree::Text(_) => tree,
            Tree::Inner {
                tag_name,
                attrs,
                children,
            } => {
                let children = children
                    .into_iter()
                    .map(|child| self.process(child))
                    .collect::<Vec<Tree>>();

                if let Some(transform) = self.transforms.get(&tag_name) {
                    self.process(transform(attrs, children))
                } else {
                    Tree::Inner {
                        tag_name,
                        attrs,
                        children,
                    }
                }
            }
        }
    }
}
