use tree_sitter::{Node, Point, Tree};

pub fn get_node_from_cursor_position(ast: &Tree, line: u32, character: u32) -> Option<Node<'_>> {
    let root = ast.root_node();

    let current_location = Point {
        row: line as usize,
        column: character as usize,
    };

    let closest_node = root.descendant_for_point_range(current_location, current_location)?;

    Some(closest_node)
}

#[cfg(test)]
mod tests {
    use super::get_node_from_cursor_position;
    use crate::parser::BladeParser;

    #[test]
    fn verify_location_at_cursor_position() {
        let test_statement = r#"<x-app-layout>
<div class="{{ str_replace() }}">
<livewire:chirps.create />
<livewire:chirps.list />
</div>
</x-app-layout>"#;

        let mut parser = BladeParser::new();
        let tree = parser.parse(test_statement, None).unwrap();
        let cur_node = get_node_from_cursor_position(&tree, 1, 21).unwrap();
        let attr = cur_node.grammar_name();
        let name = cur_node.utf8_text(test_statement.as_bytes()).unwrap();

        assert_eq!(attr, "php_only");
        assert_eq!(name, "str_replace()")
    }
}
