use super::*;

pub(crate) struct Executor;

impl Executor {
  pub(crate) fn execute<'a>(
    node: &Node<'a>,
    selector: &Expression,
  ) -> Vec<Node<'a>> {
    match selector {
      Expression::Child { parent, child } => Self::child(node, parent, child),
      Expression::Current => vec![*node],
      Expression::Descendant {
        ancestor,
        descendant,
      } => Self::descendant(node, ancestor, descendant),
      Expression::DirectChild(inner) => Self::direct_child(node, inner),
      Expression::Index { query, index } => Self::index(node, query, *index),
      Expression::Kind(kind) => Self::kind(node, kind),
      Expression::Parent { child, parent } => Self::parent(node, child, parent),
      Expression::Position(pos) => Self::position(node, *pos),
      Expression::Union(selectors) => Self::union(node, selectors),
    }
  }

  fn child<'a>(
    node: &Node<'a>,
    parent: &Expression,
    child: &Expression,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    let parent_matches = Self::execute(node, parent);

    for parent_node in parent_matches {
      results.extend(Self::direct_children(&parent_node, child));
    }

    results
  }

  fn descendant<'a>(
    node: &Node<'a>,
    ancestor: &Expression,
    descendant: &Expression,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    let ancestor_matches = Self::execute(node, ancestor);

    for ancestor_node in ancestor_matches {
      results.extend(Self::descendants(&ancestor_node, descendant));
    }

    results
  }

  fn direct_child<'a>(node: &Node<'a>, inner: &Expression) -> Vec<Node<'a>> {
    Self::direct_children(node, inner)
  }

  fn index<'a>(
    node: &Node<'a>,
    query: &Expression,
    index: usize,
  ) -> Vec<Node<'a>> {
    let matches = Self::execute(node, query);

    if let Some(result) = matches.get(index) {
      vec![*result]
    } else {
      Vec::new()
    }
  }

  fn kind<'a>(node: &Node<'a>, kind: &str) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    if node.kind() == kind {
      results.push(*node);
    }

    Self::traverse_children(node, &mut |child| {
      if child.kind() == kind {
        results.push(child);
      }
    });

    results
  }

  fn position<'a>(node: &Node<'a>, position: usize) -> Vec<Node<'a>> {
    let Ok(position) = position.try_into() else {
      return Vec::new();
    };

    if let Some(child) = node.child(position) {
      vec![child]
    } else {
      Vec::new()
    }
  }

  fn parent<'a>(
    node: &Node<'a>,
    child: &Expression,
    parent: &Expression,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    let child_matches = Self::execute(node, child);

    for child_node in child_matches {
      if let Some(parent_node) = child_node.parent() {
        let parent_results = Self::execute(&parent_node, parent);

        if !parent_results.is_empty() && parent_results[0] == parent_node {
          results.push(parent_node);
        }
      }
    }

    Self::deduplicate_and_sort(results)
  }

  fn union<'a>(node: &Node<'a>, selectors: &[Expression]) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    for selector in selectors {
      results.extend(Self::execute(node, selector));
    }

    results
  }

  fn deduplicate_and_sort<'a>(mut nodes: Vec<Node<'a>>) -> Vec<Node<'a>> {
    nodes.sort_by_key(|n| (n.start_byte(), n.end_byte()));
    nodes.dedup_by_key(|n| (n.start_byte(), n.end_byte()));
    nodes
  }

  fn child_count(node: &Node) -> u32 {
    node.child_count().try_into().unwrap_or(u32::MAX)
  }

  fn direct_children<'a>(
    node: &Node<'a>,
    selector: &Expression,
  ) -> Vec<Node<'a>> {
    if let Expression::Index { query, index } = selector {
      return Self::direct_children(node, query)
        .get(*index)
        .copied()
        .into_iter()
        .collect();
    }

    let mut results = Vec::new();

    for i in 0..Self::child_count(node) {
      if let Some(child) = node.child(i) {
        let child_results = Self::execute(&child, selector);

        for result in child_results {
          if result == child {
            results.push(result);
          }
        }
      }
    }

    results
  }

  fn descendants<'a>(node: &Node<'a>, selector: &Expression) -> Vec<Node<'a>> {
    if let Expression::Index { query, index } = selector {
      return Self::descendants(node, query)
        .get(*index)
        .copied()
        .into_iter()
        .collect();
    }

    Self::execute(node, selector)
      .into_iter()
      .filter(|result| result != node)
      .collect()
  }

  fn traverse_children<'a, F>(node: &Node<'a>, callback: &mut F)
  where
    F: FnMut(Node<'a>),
  {
    for i in 0..Self::child_count(node) {
      if let Some(child) = node.child(i) {
        callback(child);
        Self::traverse_children(&child, callback);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    crate::{lexer::Lexer, parser::Parser},
    indoc::indoc,
    tree_sitter::Tree,
  };

  fn parse(query: &str) -> Expression {
    Parser::parse(Lexer::lex(query).unwrap()).unwrap()
  }

  fn tree(input: &str) -> Tree {
    let mut parser = tree_sitter::Parser::new();

    parser
      .set_language(&tree_sitter_rust::LANGUAGE.into())
      .unwrap();

    parser.parse(input, None).unwrap()
  }

  #[test]
  fn query_by_kind() {
    let program = indoc! {"
      fn main() {
        let x = 5;
        let y = 10;
        println!(\"{}\", x + y);
      }
    "};

    let tree = tree(program);

    let nodes = Executor::execute(&tree.root_node(), &parse("function_item"));

    assert_eq!(nodes.first().unwrap().start_byte(), 0);
  }

  #[test]
  fn query_by_index() {
    let program = indoc! {"
      fn main() {
        let x = 5;
        let y = 10;
        println!(\"{}\", x + y);
      }
    "};

    let tree = tree(program);

    let nodes =
      Executor::execute(&tree.root_node(), &parse("let_declaration[0]"));

    let text = nodes
      .first()
      .unwrap()
      .utf8_text(program.as_bytes())
      .unwrap();

    assert_eq!(text, "let x = 5;");

    let nodes =
      Executor::execute(&tree.root_node(), &parse("let_declaration[1]"));

    let text = nodes
      .first()
      .unwrap()
      .utf8_text(program.as_bytes())
      .unwrap();

    assert_eq!(text, "let y = 10;");

    let nodes =
      Executor::execute(&tree.root_node(), &parse("let_declaration[2]"));

    assert_eq!(nodes.len(), 0);
  }

  #[test]
  fn query_direct_child_by_index() {
    let program = indoc! {"
      fn first() {}
      fn second() {}
    "};

    let tree = tree(program);

    for query in ["^function_item[1]", "source_file > function_item[1]"] {
      let nodes = Executor::execute(&tree.root_node(), &parse(query));

      assert_eq!(nodes.len(), 1);
      assert_eq!(
        nodes[0].utf8_text(program.as_bytes()).unwrap(),
        "fn second() {}"
      );
    }
  }

  #[test]
  fn query_descendant_excludes_ancestor() {
    let program = indoc! {"
      fn outer() {
        fn inner() {}
      }
    "};

    let tree = tree(program);

    for query in [
      "function_item function_item",
      "function_item function_item[0]",
    ] {
      let nodes = Executor::execute(&tree.root_node(), &parse(query));

      assert_eq!(nodes.len(), 1);
      assert_eq!(
        nodes[0].utf8_text(program.as_bytes()).unwrap(),
        "fn inner() {}"
      );
    }
  }

  #[test]
  fn query_group_by_index() {
    let program = indoc! {"
      const VALUE: usize = 0;
      fn first() {}
      fn second() {}
    "};

    let tree = tree(program);

    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("(const_item, function_item)[1]"),
    );

    assert_eq!(nodes.len(), 1);
    assert_eq!(
      nodes[0].utf8_text(program.as_bytes()).unwrap(),
      "fn first() {}"
    );
  }

  #[test]
  fn query_hierarchy_by_index() {
    let program = indoc! {"
      fn first() {}
      fn second() {}
    "};

    let tree = tree(program);

    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("(source_file > function_item)[1]"),
    );

    assert_eq!(nodes.len(), 1);
    assert_eq!(
      nodes[0].utf8_text(program.as_bytes()).unwrap(),
      "fn second() {}"
    );
  }

  #[test]
  fn query_parent() {
    let program = indoc! {"
      fn main() {
        let x = 5;
        let y = 10;
        println!(\"{}\", x + y);
      }
    "};

    let tree = tree(program);

    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("identifier < let_declaration"),
    );

    assert_eq!(nodes.len(), 2);

    assert_eq!(
      nodes[0].utf8_text(program.as_bytes()).unwrap(),
      "let x = 5;"
    );

    assert_eq!(
      nodes[1].utf8_text(program.as_bytes()).unwrap(),
      "let y = 10;"
    );
  }

  #[test]
  fn query_child_with_implicit_current() {
    let program = indoc! {"
      fn main() {
        let x = 5;
      }
    "};

    let tree = tree(program);

    let block_nodes = Executor::execute(&tree.root_node(), &parse("block"));

    assert_eq!(block_nodes.len(), 1);

    let nodes = Executor::execute(
      block_nodes.first().unwrap(),
      &parse("> let_declaration"),
    );

    assert_eq!(nodes.len(), 1);

    assert_eq!(
      nodes[0].utf8_text(program.as_bytes()).unwrap(),
      "let x = 5;"
    );
  }

  #[test]
  fn query_parent_with_implicit_current() {
    let program = indoc! {"
      fn main() {
        let x = 5;
      }
    "};

    let tree = tree(program);

    let let_nodes =
      Executor::execute(&tree.root_node(), &parse("let_declaration"));

    assert_eq!(let_nodes.len(), 1);

    let nodes =
      Executor::execute(let_nodes.first().unwrap(), &parse("< block"));

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].kind(), "block");
  }

  #[test]
  fn query_union_preserves_selector_order() {
    let program = indoc! {"
      const VALUE: usize = 0;
      fn first() {}
      fn second() {}
    "};

    let tree = tree(program);

    let nodes =
      Executor::execute(&tree.root_node(), &parse("function_item, const_item"));

    assert_eq!(
      nodes.iter().map(Node::kind).collect::<Vec<_>>(),
      ["function_item", "function_item", "const_item"]
    );
  }
}
