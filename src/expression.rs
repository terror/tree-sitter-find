#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expression {
  Child {
    child: Box<Expression>,
    parent: Box<Expression>,
  },
  Current,
  Descendant {
    ancestor: Box<Expression>,
    descendant: Box<Expression>,
  },
  DirectChild(Box<Expression>),
  Index {
    index: usize,
    query: Box<Expression>,
  },
  Kind(String),
  Parent {
    child: Box<Expression>,
    parent: Box<Expression>,
  },
  Position(usize),
  Union(Vec<Expression>),
}
