extern crate cc;

fn main() {
    cc::Build::new()
        .file("./deps/src/parser.c")
        .include("./deps/src")
        .compile("tree_sitter_blade");
}
