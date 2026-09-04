use super::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
/// An error encountered while parsing a query.
pub enum Error {
  /// An index expression does not contain a numeric index.
  #[snafu(display("invalid index in query"))]
  InvalidIndex,
  /// A numeric index cannot be represented as a [`usize`].
  #[snafu(display("failed to parse number: {}", source))]
  ParseNumber {
    /// The integer parsing error.
    source: std::num::ParseIntError,
  },
  /// The query ended before an expression was complete.
  #[snafu(display("unexpected end of input"))]
  UnexpectedEnd,
  /// The query contains a character that is not part of the language.
  #[snafu(display("unexpected character `{character}` at byte {position}"))]
  UnexpectedCharacter {
    /// The unexpected character.
    character: char,
    /// The byte position of the unexpected character.
    position: usize,
  },
  /// The parser encountered a token that is not valid in its position.
  #[snafu(display("unexpected token in query"))]
  UnexpectedToken,
}
