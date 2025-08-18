use super::*;

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Invalid index in query"))]
  InvalidIndex,
  #[snafu(display("Unexpected end of input"))]
  UnexpectedEnd,
  #[snafu(display("Unexpected token in query"))]
  UnexpectedToken,
}
