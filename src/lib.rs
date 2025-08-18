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

#[cfg(test)]
#[macro_export]
macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}

pub use error::Error;

pub type Result<T = (), E = error::Error> = std::result::Result<T, E>;

pub trait NodeExt {
  fn find(&self, query: &str) -> Result<Vec<Node<'_>>>;
}

impl NodeExt for Node<'_> {
  fn find(&self, query: &str) -> Result<Vec<Node<'_>>> {
    Ok(Executor::execute(self, &Parser::parse(Lexer::lex(query)?)?))
  }
}
