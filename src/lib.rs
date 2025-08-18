use {
  executor::Executor, lexer::Lexer, parser::Parser, query::Query,
  snafu::prelude::*, token::Token, tree_sitter::Node,
};

mod error;
mod executor;
mod lexer;
mod parser;
mod query;
mod token;

pub use error::Error;

pub type Result<T = (), E = error::Error> = std::result::Result<T, E>;

pub trait NodeExt {
  fn find(&self, query: &str) -> Result<Vec<Node<'_>>>;
}

impl NodeExt for Node<'_> {
  fn find(&self, query: &str) -> Result<Vec<Node<'_>>> {
    Ok(Executor::execute(self, &Parser::parse(Lexer::lex(query))?))
  }
}
