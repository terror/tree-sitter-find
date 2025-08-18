#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
  At,
  Caret,
  Comma,
  Greater,
  Kind(String),
  LeftBracket,
  Number(usize),
  RightBracket,
}
