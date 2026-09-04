use {
  expression::Expression, snafu::prelude::*, token::Token, tree_sitter::Node,
};

mod error;
mod executor;
mod expression;
mod lexer;
mod parser;
mod query;
mod query_input;
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

pub use {error::Error, query::Query, query_input::QueryInput};

pub type Result<T = (), E = error::Error> = std::result::Result<T, E>;

pub trait NodeExt<'tree> {
  fn find<'query>(
    &self,
    query: impl Into<QueryInput<'query>>,
  ) -> Result<Vec<Node<'tree>>>;
}

impl<'tree> NodeExt<'tree> for Node<'tree> {
  fn find<'query>(
    &self,
    query: impl Into<QueryInput<'query>>,
  ) -> Result<Vec<Node<'tree>>> {
    query.into().execute(self)
  }
}
