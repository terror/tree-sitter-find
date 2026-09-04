#![warn(missing_docs)]

//! A compact query language for finding nodes in tree-sitter syntax trees.
//!
//! Queries can select nodes by kind (`identifier`), child position (`@0`), or
//! zero-based result index (`identifier[0]`). Use whitespace for descendants,
//! `>` for direct children, `<` for parents, `^` for direct children of the
//! input node, commas for unions, and parentheses for grouping.

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
#[doc(hidden)]
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

/// A result returned by `tree-sitter-find`.
pub type Result<T = (), E = error::Error> = std::result::Result<T, E>;

/// Adds query-based node discovery to tree-sitter nodes.
pub trait NodeExt<'tree> {
  /// Finds nodes selected by `query`.
  ///
  /// The query may be source text or a precompiled [`Query`].
  ///
  /// # Errors
  ///
  /// Returns an error when query source cannot be parsed. Executing a
  /// precompiled query does not fail.
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
