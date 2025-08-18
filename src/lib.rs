use {
  executor::Executor, lexer::Lexer, parser::Parser, query::Query, token::Token,
  tree_sitter::Node,
};

mod executor;
mod lexer;
mod parser;
mod query;
mod token;

pub use parser::ParseError;

pub trait NodeExt {
  fn find(&self, query: &str) -> Result<Vec<Node<'_>>, ParseError>;
}

impl NodeExt for Node<'_> {
  fn find(&self, query: &str) -> Result<Vec<Node<'_>>, ParseError> {
    Ok(Executor::execute(self, &Parser::parse(Lexer::lex(query))?))
  }
}
