use super::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
  #[snafu(display("invalid index in query"))]
  InvalidIndex,
  #[snafu(display("failed to parse number: {}", source))]
  ParseNumber { source: std::num::ParseIntError },
  #[snafu(display("unexpected end of input"))]
  UnexpectedEnd,
  #[snafu(display("unexpected character `{character}` at byte {position}"))]
  UnexpectedCharacter { character: char, position: usize },
  #[snafu(display("unexpected token in query"))]
  UnexpectedToken,
}
