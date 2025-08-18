#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Query {
  Child {
    child: Box<Query>,
    parent: Box<Query>,
  },
  Current,
  Descendant {
    ancestor: Box<Query>,
    descendant: Box<Query>,
  },
  DirectChild(Box<Query>),
  Index {
    index: usize,
    kind: String,
  },
  Kind(String),
  Parent {
    child: Box<Query>,
    parent: Box<Query>,
  },
  Position(usize),
  Union(Vec<Query>),
}
