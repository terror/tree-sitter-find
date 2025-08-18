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

  fn current_token(&self) -> Option<&Token> {
    self.input.get(self.position)
  }

  fn advance(&mut self) -> Option<Token> {
    let token = self.current_token().cloned();
    self.position += 1;
    token
  }

  pub(crate) fn parse(input: Vec<Token>) -> Result<Selector, ParseError> {
    let mut parser = Self::new(input);
    parser.parse_union_selector()
  }

  fn parse_union_selector(&mut self) -> Result<Selector, ParseError> {
    let mut selectors = vec![self.parse_hierarchical_selector()?];

    while let Some(Token::Comma) = self.current_token() {
      self.advance();
      selectors.push(self.parse_hierarchical_selector()?);
    }

    if selectors.len() == 1 {
      Ok(selectors.into_iter().next().unwrap())
    } else {
      Ok(Selector::Union(selectors))
    }
  }

  fn parse_hierarchical_selector(&mut self) -> Result<Selector, ParseError> {
    let mut left = self.parse_simple_selector()?;

    loop {
      match self.current_token() {
        Some(Token::Greater) => {
          self.advance();

          let right = self.parse_simple_selector()?;

          left = Selector::Child {
            parent: Box::new(left),
            child: Box::new(right),
          };
        }
        Some(Token::Kind(_)) | Some(Token::Caret) | Some(Token::At) => {
          let right = self.parse_simple_selector()?;

          left = Selector::Descendant {
            ancestor: Box::new(left),
            descendant: Box::new(right),
          };
        }
        _ => break,
      }
    }

    Ok(left)
  }

  fn parse_simple_selector(&mut self) -> Result<Selector, ParseError> {
    match self.current_token() {
      Some(Token::Caret) => {
        self.advance();
        let inner = self.parse_simple_selector()?;
        Ok(Selector::DirectChild(Box::new(inner)))
      }
      Some(Token::At) => {
        self.advance();

        if let Some(Token::Number(pos)) = self.advance() {
          Ok(Selector::Position(pos))
        } else {
          Err(ParseError::UnexpectedToken)
        }
      }
      Some(Token::Kind(kind)) => {
        let kind = kind.clone();

        self.advance();

        if let Some(Token::LeftBracket) = self.current_token() {
          self.advance();

          if let Some(Token::Number(index)) = self.advance() {
            if let Some(Token::RightBracket) = self.advance() {
              Ok(Selector::Index { kind, index })
            } else {
              Err(ParseError::UnexpectedToken)
            }
          } else {
            Err(ParseError::InvalidIndex)
          }
        } else {
          Ok(Selector::Kind(kind))
        }
      }
      _ => Err(ParseError::UnexpectedToken),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn selector<'a>(input: &'a str) -> Selector {
    Parser::parse(Lexer::lex(input)).unwrap()
  }

  #[test]
  fn child() {
    assert_eq!(
      selector("object > string"),
      Selector::Child {
        child: Box::new(Selector::Kind("string".to_string())),
        parent: Box::new(Selector::Kind("object".to_string()))
      }
    );
  }

  #[test]
  fn descendant() {
    assert_eq!(
      selector("object string"),
      Selector::Descendant {
        ancestor: Box::new(Selector::Kind("object".to_string())),
        descendant: Box::new(Selector::Kind("string".to_string()))
      }
    );
  }

  #[test]
  fn direct_child() {
    assert_eq!(
      selector("^string"),
      Selector::DirectChild(Box::new(Selector::Kind("string".to_string())))
    );
  }

  #[test]
  fn index() {
    assert_eq!(
      selector("string[2]"),
      Selector::Index {
        index: 2,
        kind: "string".to_string(),
      }
    );
  }

  #[test]
  fn kind() {
    assert_eq!(selector("string"), Selector::Kind("string".to_string()));
  }

  #[test]
  fn position() {
    assert_eq!(selector("@1"), Selector::Position(1));
  }

  #[test]
  fn union() {
    assert_eq!(
      selector("string, number"),
      Selector::Union(vec![
        Selector::Kind("string".to_string()),
        Selector::Kind("number".to_string())
      ])
    );
  }
}
