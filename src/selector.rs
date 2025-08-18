#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Selector {
  Child {
    child: Box<Selector>,
    parent: Box<Selector>,
  },
  Descendant {
    ancestor: Box<Selector>,
    descendant: Box<Selector>,
  },
  DirectChild(Box<Selector>),
  Index {
    index: usize,
    kind: String,
  },
  Kind(String),
  Position(usize),
  Union(Vec<Selector>),
}
