use super::*;

pub(crate) struct Executor;

impl Executor {
  pub(crate) fn execute<'a>(
    node: &Node<'a>,
    selector: &Query,
  ) -> Vec<Node<'a>> {
    match selector {
      Query::Child { parent, child } => Self::find_child(node, parent, child),
      Query::Descendant {
        ancestor,
        descendant,
      } => Self::find_descendant(node, ancestor, descendant),
      Query::DirectChild(inner) => Self::find_direct_child(node, inner),
      Query::Index { kind, index } => Self::find_by_index(node, kind, *index),
      Query::Kind(kind) => Self::find_by_kind(node, kind),
      Query::Position(pos) => Self::find_by_position(node, *pos),
      Query::Union(selectors) => Self::find_union(node, selectors),
    }
  }

  fn find_by_kind<'a>(node: &Node<'a>, kind: &str) -> Vec<Node<'a>> {
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

  fn find_by_index<'a>(
    node: &Node<'a>,
    kind: &str,
    index: usize,
  ) -> Vec<Node<'a>> {
    let matches = Self::find_by_kind(node, kind);

    if let Some(result) = matches.get(index) {
      vec![*result]
    } else {
      Vec::new()
    }
  }

  fn find_by_position<'a>(node: &Node<'a>, position: usize) -> Vec<Node<'a>> {
    if let Some(child) = node.child(position) {
      vec![child]
    } else {
      Vec::new()
    }
  }

  fn find_direct_child<'a>(node: &Node<'a>, inner: &Query) -> Vec<Node<'a>> {
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

  fn find_child<'a>(
    node: &Node<'a>,
    parent: &Query,
    child: &Query,
  ) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    let parent_matches = Self::execute(node, parent);

    for parent_node in parent_matches {
      results.extend(Self::find_direct_children(&parent_node, child));
    }

    results
  }

  fn find_direct_children<'a>(
    node: &Node<'a>,
    selector: &Query,
  ) -> Vec<Node<'a>> {
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

  fn find_descendant<'a>(
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

  fn find_union<'a>(node: &Node<'a>, selectors: &[Query]) -> Vec<Node<'a>> {
    let mut results = Vec::new();

    for selector in selectors {
      results.extend(Self::execute(node, selector));
    }

    Self::deduplicate_and_sort(results)
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

  fn deduplicate_and_sort<'a>(mut nodes: Vec<Node<'a>>) -> Vec<Node<'a>> {
    nodes.sort_by_key(|n| (n.start_byte(), n.end_byte()));
    nodes.dedup_by_key(|n| (n.start_byte(), n.end_byte()));
    nodes
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
}
