use super::*;

#[derive(Debug)]
pub enum ParseError {
  InvalidIndex,
  UnexpectedEnd,
  UnexpectedToken,
}

pub(crate) struct Parser {
  input: Vec<Token>,
  position: usize,
}

impl Parser {
  pub(crate) fn new(tokens: Vec<Token>) -> Self {
    Self {
      input: tokens,
      position: 0,
    }
  }

  fn advance(&mut self) -> Option<Token> {
    let token = self.token().cloned();
    self.position += 1;
    token
  }

  fn token(&self) -> Option<&Token> {
    self.input.get(self.position)
  }

  pub(crate) fn parse(input: Vec<Token>) -> Result<Query, ParseError> {
    Self::new(input).parse_union_query()
  }

  fn parse_union_query(&mut self) -> Result<Query, ParseError> {
    let mut selectors = vec![self.parse_hierarchical_query()?];

    while let Some(Token::Comma) = self.token() {
      self.advance();
      selectors.push(self.parse_hierarchical_query()?);
    }

    if selectors.len() == 1 {
      Ok(selectors.into_iter().next().unwrap())
    } else {
      Ok(Query::Union(selectors))
    }
  }

  fn parse_hierarchical_query(&mut self) -> Result<Query, ParseError> {
    let mut left = self.parse_simple_query()?;

    loop {
      match self.token() {
        Some(Token::Greater) => {
          self.advance();

          left = Query::Child {
            parent: Box::new(left),
            child: Box::new(self.parse_simple_query()?),
          };
        }
        Some(Token::Kind(_)) | Some(Token::Caret) | Some(Token::At) => {
          left = Query::Descendant {
            ancestor: Box::new(left),
            descendant: Box::new(self.parse_simple_query()?),
          };
        }
        _ => break,
      }
    }

    Ok(left)
  }

  fn parse_simple_query(&mut self) -> Result<Query, ParseError> {
    match self.token() {
      Some(Token::Caret) => {
        self.advance();
        Ok(Query::DirectChild(Box::new(self.parse_simple_query()?)))
      }
      Some(Token::At) => {
        self.advance();

        if let Some(Token::Number(position)) = self.advance() {
          Ok(Query::Position(position))
        } else {
          Err(ParseError::UnexpectedToken)
        }
      }
      Some(Token::Kind(kind)) => {
        let kind = kind.clone();

        self.advance();

        if let Some(Token::LeftBracket) = self.token() {
          self.advance();

          if let Some(Token::Number(index)) = self.advance() {
            if let Some(Token::RightBracket) = self.advance() {
              Ok(Query::Index { kind, index })
            } else {
              Err(ParseError::UnexpectedToken)
            }
          } else {
            Err(ParseError::InvalidIndex)
          }
        } else {
          Ok(Query::Kind(kind))
        }
      }
      _ => Err(ParseError::UnexpectedToken),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn query<'a>(input: &'a str) -> Query {
    Parser::parse(Lexer::lex(input)).unwrap()
  }

  #[test]
  fn child() {
    assert_eq!(
      query("object > string"),
      Query::Child {
        child: Box::new(Query::Kind("string".to_string())),
        parent: Box::new(Query::Kind("object".to_string()))
      }
    );
  }

  #[test]
  fn descendant() {
    assert_eq!(
      query("object string"),
      Query::Descendant {
        ancestor: Box::new(Query::Kind("object".to_string())),
        descendant: Box::new(Query::Kind("string".to_string()))
      }
    );
  }

  #[test]
  fn direct_child() {
    assert_eq!(
      query("^string"),
      Query::DirectChild(Box::new(Query::Kind("string".to_string())))
    );
  }

  #[test]
  fn index() {
    assert_eq!(
      query("string[2]"),
      Query::Index {
        index: 2,
        kind: "string".to_string(),
      }
    );
  }

  #[test]
  fn kind() {
    assert_eq!(query("string"), Query::Kind("string".to_string()));
  }

  #[test]
  fn position() {
    assert_eq!(query("@1"), Query::Position(1));
  }

  #[test]
  fn union() {
    assert_eq!(
      query("string, number"),
      Query::Union(vec![
        Query::Kind("string".to_string()),
        Query::Kind("number".to_string())
      ])
    );
  }

  #[test]
  fn union_query_with_descendant_and_direct_child() {
    assert_eq!(
      query("object > array[0] string @ 2, ^number > boolean"),
      Query::Union(vec![
        Query::Descendant {
          ancestor: Box::new(Query::Descendant {
            ancestor: Box::new(Query::Child {
              parent: Box::new(Query::Kind("object".to_string())),
              child: Box::new(Query::Index {
                kind: "array".to_string(),
                index: 0,
              }),
            }),
            descendant: Box::new(Query::Kind("string".to_string())),
          }),
          descendant: Box::new(Query::Position(2)),
        },
        Query::Child {
          parent: Box::new(Query::DirectChild(Box::new(Query::Kind(
            "number".to_string()
          )))),
          child: Box::new(Query::Kind("boolean".to_string())),
        },
      ])
    );
  }
}
