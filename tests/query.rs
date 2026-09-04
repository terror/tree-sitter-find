use {
  tree_sitter::{Parser, Tree},
  tree_sitter_find::{NodeExt, Query},
};

fn tree(input: &str) -> Tree {
  let mut parser = Parser::new();

  parser
    .set_language(&tree_sitter_rust::LANGUAGE.into())
    .unwrap();

  parser.parse(input, None).unwrap()
}

#[test]
fn compiled_query_can_be_reused() {
  let query: Query = "function_item[0]".parse().unwrap();

  for input in ["fn first() {}", "fn second() {}"] {
    let tree = tree(input);
    let nodes = tree.root_node().find(&query).unwrap();

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].utf8_text(input.as_bytes()).unwrap(), input);
  }
}

#[test]
fn node_extension_accepts_query_source() {
  let input = "fn first() {} fn second() {}";
  let tree = tree(input);

  let nodes = tree
    .root_node()
    .find("(source_file > function_item)[1]")
    .unwrap();

  assert_eq!(nodes.len(), 1);
  assert_eq!(
    nodes[0].utf8_text(input.as_bytes()).unwrap(),
    "fn second() {}"
  );
}
