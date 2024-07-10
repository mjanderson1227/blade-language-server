use std::ops::{Deref, DerefMut};

#[allow(unused_imports)]
use tree_sitter::{InputEdit, Language, Parser, Point, Tree};

extern "C" {
    fn tree_sitter_blade() -> Language;
}

pub enum Location {
    Php,
    Markup,
    Directive,
    Tailwind,
}

pub struct BladeParser {
    parser: Parser,
}

impl BladeParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();

        unsafe {
            if !parser.set_language(&tree_sitter_blade()).is_ok() {
                panic!("Error occurred while trying to load the blade treesitter library");
            }
        }

        Self { parser }
    }
}

impl Deref for BladeParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for BladeParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}
