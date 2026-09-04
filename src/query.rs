use {
  crate::{
    Error, Result, executor::Executor, expression::Expression, lexer::Lexer,
    parser::Parser,
  },
  std::str::FromStr,
  tree_sitter::Node,
};

#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct Query {
  expression: Expression,
}

impl Query {
  pub(crate) fn execute<'tree>(&self, node: &Node<'tree>) -> Vec<Node<'tree>> {
    Executor::execute(node, &self.expression)
  }

  pub fn parse(input: &str) -> Result<Self> {
    Ok(Self {
      expression: Parser::parse(Lexer::lex(input)?)?,
    })
  }
}

impl FromStr for Query {
  type Err = Error;

  fn from_str(input: &str) -> Result<Self> {
    Self::parse(input)
  }
}
