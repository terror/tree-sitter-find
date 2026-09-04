use super::*;

pub(crate) struct Executor;

#[derive(Clone, Copy)]
enum Scope {
  Descendants,
  DirectChildren,
  Inclusive,
  NodeOnly,
}

impl Executor {
  pub(crate) fn execute<'a>(
    node: &Node<'a>,
    selector: &Expression,
  ) -> Vec<Node<'a>> {
    Self::execute_in(node, selector, Scope::Inclusive)
  }

  fn execute_in<'a>(
    node: &Node<'a>,
    selector: &Expression,
    scope: Scope,
  ) -> Vec<Node<'a>> {
    match selector {
      Expression::Child { parent, child } => {
        Self::child(node, parent, child, scope)
      }
      Expression::Current => vec![*node],
      Expression::Descendant {
        ancestor,
        descendant,
      } => Self::descendant(node, ancestor, descendant, scope),
      Expression::DirectChild(inner) => Self::direct_child(node, inner),
      Expression::Index { query, index } => {
        Self::index(node, query, *index, scope)
      }
      Expression::Kind(kind) => Self::kind(node, kind, scope),
      Expression::Parent { child, parent } => {
        Self::parent(node, child, parent, scope)
      }
      Expression::Position(pos) => Self::position(node, *pos),
      Expression::Union(selectors) => Self::union(node, selectors, scope),
    }
  }

  fn child<'a>(
    node: &Node<'a>,
    parent: &Expression,
    child: &Expression,
    scope: Scope,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    let parent_matches = Self::execute_in(node, parent, scope);

    for parent_node in parent_matches {
      results.extend(Self::direct_children(&parent_node, child));
    }

    results
  }

  fn descendant<'a>(
    node: &Node<'a>,
    ancestor: &Expression,
    descendant: &Expression,
    scope: Scope,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    let ancestor_matches = Self::execute_in(node, ancestor, scope);

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
    scope: Scope,
  ) -> Vec<Node<'a>> {
    Self::execute_in(node, query, scope)
      .into_iter()
      .filter(|result| Self::in_scope(node, result, scope))
      .nth(index)
      .into_iter()
      .collect()
  }

  fn kind<'a>(node: &Node<'a>, kind: &str, scope: Scope) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    if matches!(scope, Scope::Inclusive | Scope::NodeOnly)
      && node.kind() == kind
    {
      results.push(*node);
    }

    match scope {
      Scope::Descendants | Scope::Inclusive => {
        Self::traverse_children(node, &mut |child| {
          if child.kind() == kind {
            results.push(child);
          }
        });
      }
      Scope::DirectChildren => {
        for position in 0..Self::child_count(node) {
          if let Some(child) = node.child(position) {
            if child.kind() == kind {
              results.push(child);
            }
          }
        }
      }
      Scope::NodeOnly => {}
    }

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
    scope: Scope,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    let child_matches = Self::execute_in(node, child, scope);

    for child_node in child_matches {
      if let Some(parent_node) = child_node.parent() {
        if Self::matches_node(&parent_node, parent) {
          results.push(parent_node);
        }
      }
    }

    Self::deduplicate_and_sort(results)
  }

  fn union<'a>(
    node: &Node<'a>,
    selectors: &[Expression],
    scope: Scope,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    for selector in selectors {
      results.extend(Self::execute_in(node, selector, scope));
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
    Self::execute_in(node, selector, Scope::DirectChildren)
      .into_iter()
      .filter(|result| Self::in_scope(node, result, Scope::DirectChildren))
      .collect()
  }

  fn descendants<'a>(node: &Node<'a>, selector: &Expression) -> Vec<Node<'a>> {
    Self::execute_in(node, selector, Scope::Descendants)
      .into_iter()
      .filter(|result| Self::in_scope(node, result, Scope::Descendants))
      .collect()
  }

  fn in_scope(root: &Node, node: &Node, scope: Scope) -> bool {
    match scope {
      Scope::Descendants => Self::is_descendant(root, node),
      Scope::DirectChildren => node.parent() == Some(*root),
      Scope::Inclusive => true,
      Scope::NodeOnly => node == root,
    }
  }

  fn is_descendant(ancestor: &Node, node: &Node) -> bool {
    let mut current = node.parent();

    while let Some(parent) = current {
      if parent == *ancestor {
        return true;
      }

      current = parent.parent();
    }

    false
  }

  fn matches_node(node: &Node, selector: &Expression) -> bool {
    Self::execute_in(node, selector, Scope::NodeOnly).contains(node)
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
  fn query_direct_child_scopes_indexed_group() {
    let program = indoc! {"
      const VALUE: usize = 0;
      fn first() {}
      fn second() {}
    "};

    let tree = tree(program);
    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("source_file > (function_item[0], const_item)"),
    );

    assert_eq!(
      nodes.iter().map(Node::kind).collect::<Vec<_>>(),
      ["function_item", "const_item"]
    );
    assert_eq!(
      nodes[0].utf8_text(program.as_bytes()).unwrap(),
      "fn first() {}"
    );
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
  fn query_descendant_scopes_indexed_group() {
    let program = indoc! {"
      fn outer() {
        fn inner() {}
      }
    "};

    let tree = tree(program);
    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("function_item (function_item[0], struct_item)"),
    );

    assert_eq!(nodes.len(), 1);
    assert_eq!(
      nodes[0].utf8_text(program.as_bytes()).unwrap(),
      "fn inner() {}"
    );
  }

  #[test]
  fn query_descendant_scopes_hierarchical_selector() {
    let program = indoc! {"
      fn outer() {
        fn inner() {}
      }
    "};

    let tree = tree(program);
    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("function_item (function_item[0] > identifier)"),
    );

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].utf8_text(program.as_bytes()).unwrap(), "inner");
  }

  #[test]
  fn query_descendant_does_not_escape_ancestor() {
    let tree = tree("fn main() {}");
    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("function_item (< source_file)"),
    );

    assert!(nodes.is_empty());
  }

  #[test]
  fn query_descendant_indexes_after_scoping() {
    let program = "fn main() {}";
    let tree = tree(program);
    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("function_item (< source_file, identifier)[0]"),
    );

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].utf8_text(program.as_bytes()).unwrap(), "main");
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
  fn query_parent_matches_any_group_branch() {
    let program = "fn main() {}";
    let tree = tree(program);

    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("identifier < (identifier, function_item)"),
    );

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].kind(), "function_item");
  }

  #[test]
  fn query_parent_scopes_index() {
    let program = "fn main() {}";
    let tree = tree(program);

    let nodes = Executor::execute(
      &tree.root_node(),
      &parse("identifier < function_item[0]"),
    );

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].utf8_text(program.as_bytes()).unwrap(), program);
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
