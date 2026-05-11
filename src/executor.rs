use super::*;

pub(crate) struct Executor;

impl Executor {
  pub(crate) fn execute<'a>(
    node: &Node<'a>,
    selector: &Query,
  ) -> Vec<Node<'a>> {
    match selector {
      Query::Child { parent, child } => Self::child(node, parent, child),
      Query::Current => vec![*node],
      Query::Descendant {
        ancestor,
        descendant,
      } => Self::descendant(node, ancestor, descendant),
      Query::DirectChild(inner) => Self::direct_child(node, inner),
      Query::Index { kind, index } => Self::index(node, kind, *index),
      Query::Kind(kind) => Self::kind(node, kind),
      Query::Parent { child, parent } => Self::parent(node, child, parent),
      Query::Position(pos) => Self::position(node, *pos),
      Query::Union(selectors) => Self::union(node, selectors),
    }
  }

  fn child<'a>(
    node: &Node<'a>,
    parent: &Query,
    child: &Query,
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
    ancestor: &Query,
    descendant: &Query,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    let ancestor_matches = Self::execute(node, ancestor);

    for ancestor_node in ancestor_matches {
      results.extend(Self::execute(&ancestor_node, descendant));
    }

    results
  }

  fn direct_child<'a>(node: &Node<'a>, inner: &Query) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        let child_results = Self::execute(&child, inner);

        if !child_results.is_empty() && child_results[0] == child {
          results.push(child);
        }
      }
    }

    results
  }

  fn index<'a>(node: &Node<'a>, kind: &str, index: usize) -> Vec<Node<'a>> {
    let matches = Self::kind(node, kind);

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
    if let Some(child) = node.child(position) {
      vec![child]
    } else {
      Vec::new()
    }
  }

  fn parent<'a>(
    node: &Node<'a>,
    child: &Query,
    parent: &Query,
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

  fn union<'a>(node: &Node<'a>, selectors: &[Query]) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    for selector in selectors {
      results.extend(Self::execute(node, selector));
    }

    Self::deduplicate_and_sort(results)
  }

  fn deduplicate_and_sort<'a>(mut nodes: Vec<Node<'a>>) -> Vec<Node<'a>> {
    nodes.sort_by_key(|n| (n.start_byte(), n.end_byte()));
    nodes.dedup_by_key(|n| (n.start_byte(), n.end_byte()));
    nodes
  }

  fn direct_children<'a>(node: &Node<'a>, selector: &Query) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    for i in 0..node.child_count() {
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

  fn traverse_children<'a, F>(node: &Node<'a>, callback: &mut F)
  where
    F: FnMut(Node<'a>),
  {
    for i in 0..node.child_count() {
      if let Some(child) = node.child(i) {
        callback(child);
        Self::traverse_children(&child, callback);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, tree_sitter::Tree};

  fn parse(query: &str) -> Query {
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
}
