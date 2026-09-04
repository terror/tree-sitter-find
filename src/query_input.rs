use {crate::*, tree_sitter::Node};

#[derive(Clone, Copy, Debug)]
enum QueryInputKind<'query> {
  Compiled(&'query Query),
  Source(&'query str),
}

#[derive(Clone, Copy, Debug)]
#[must_use]
/// Query input accepted by [`NodeExt::find`].
///
/// Values are created automatically from query source or a compiled
/// [`Query`].
pub struct QueryInput<'query> {
  kind: QueryInputKind<'query>,
}

impl QueryInput<'_> {
  pub(crate) fn execute<'tree>(
    self,
    node: &Node<'tree>,
  ) -> Result<Vec<Node<'tree>>> {
    match self.kind {
      QueryInputKind::Compiled(query) => Ok(query.execute(node)),
      QueryInputKind::Source(source) => {
        Query::parse(source).map(|query| query.execute(node))
      }
    }
  }
}

impl<'query> From<&'query Query> for QueryInput<'query> {
  fn from(query: &'query Query) -> Self {
    Self {
      kind: QueryInputKind::Compiled(query),
    }
  }
}

impl<'query> From<&'query String> for QueryInput<'query> {
  fn from(query: &'query String) -> Self {
    Self {
      kind: QueryInputKind::Source(query),
    }
  }
}

impl<'query> From<&'query str> for QueryInput<'query> {
  fn from(query: &'query str) -> Self {
    Self {
      kind: QueryInputKind::Source(query),
    }
  }
}
