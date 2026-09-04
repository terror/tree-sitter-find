use super::*;

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

  pub(crate) fn parse(input: Vec<Token>) -> Result<Expression, Error> {
    let mut parser = Self::new(input);
    let query = parser.parse_union_query()?;

    if parser.token().is_some() {
      Err(Error::UnexpectedToken)
    } else {
      Ok(query)
    }
  }

  fn parse_union_query(&mut self) -> Result<Expression, Error> {
    let mut selectors = vec![self.parse_hierarchical_query()?];

    while let Some(Token::Comma) = self.token() {
      self.advance();
      selectors.push(self.parse_hierarchical_query()?);
    }

    if selectors.len() == 1 {
      Ok(selectors.into_iter().next().unwrap())
    } else {
      Ok(Expression::Union(selectors))
    }
  }

  fn parse_hierarchical_query(&mut self) -> Result<Expression, Error> {
    let mut left = match self.token() {
      Some(Token::Greater) => {
        self.advance();

        Expression::Child {
          parent: Box::new(Expression::Current),
          child: Box::new(self.parse_postfix_query()?),
        }
      }
      Some(Token::Less) => {
        self.advance();

        Expression::Parent {
          child: Box::new(Expression::Current),
          parent: Box::new(self.parse_postfix_query()?),
        }
      }
      _ => self.parse_postfix_query()?,
    };

    loop {
      match self.token() {
        Some(Token::Greater) => {
          self.advance();

          left = Expression::Child {
            parent: Box::new(left),
            child: Box::new(self.parse_postfix_query()?),
          };
        }
        Some(Token::Less) => {
          self.advance();

          left = Expression::Parent {
            child: Box::new(left),
            parent: Box::new(self.parse_postfix_query()?),
          };
        }
        Some(Token::Kind(_))
        | Some(Token::Caret)
        | Some(Token::At)
        | Some(Token::LeftParen) => {
          left = Expression::Descendant {
            ancestor: Box::new(left),
            descendant: Box::new(self.parse_postfix_query()?),
          };
        }
        _ => break,
      }
    }

    Ok(left)
  }

  fn parse_postfix_query(&mut self) -> Result<Expression> {
    let mut query = self.parse_primary_query()?;

    while let Some(Token::LeftBracket) = self.token() {
      self.advance();

      let index = match self.advance() {
        Some(Token::Number(index)) => index,
        Some(_) => return Err(Error::InvalidIndex),
        None => return Err(Error::UnexpectedEnd),
      };

      match self.advance() {
        Some(Token::RightBracket) => {}
        Some(_) => return Err(Error::UnexpectedToken),
        None => return Err(Error::UnexpectedEnd),
      }

      query = Expression::Index {
        index,
        query: Box::new(query),
      };
    }

    Ok(query)
  }

  fn parse_primary_query(&mut self) -> Result<Expression> {
    match self.token() {
      Some(Token::Caret) => {
        self.advance();
        Ok(Expression::DirectChild(Box::new(
          self.parse_postfix_query()?,
        )))
      }
      Some(Token::At) => {
        self.advance();

        match self.advance() {
          Some(Token::Number(position)) => Ok(Expression::Position(position)),
          Some(_) => Err(Error::UnexpectedToken),
          None => Err(Error::UnexpectedEnd),
        }
      }
      Some(Token::Kind(kind)) => {
        let kind = kind.clone();

        self.advance();
        Ok(Expression::Kind(kind))
      }
      Some(Token::LeftParen) => {
        self.advance();

        let query = self.parse_union_query()?;

        match self.advance() {
          Some(Token::RightParen) => Ok(query),
          Some(_) => Err(Error::UnexpectedToken),
          None => Err(Error::UnexpectedEnd),
        }
      }
      Some(_) => Err(Error::UnexpectedToken),
      None => Err(Error::UnexpectedEnd),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, crate::lexer::Lexer};

  fn query(input: &str) -> Expression {
    Parser::parse(Lexer::lex(input).unwrap()).unwrap()
  }

  #[test]
  fn child() {
    assert_eq!(
      query("object > string"),
      Expression::Child {
        child: Box::new(Expression::Kind("string".to_string())),
        parent: Box::new(Expression::Kind("object".to_string()))
      }
    );
  }

  #[test]
  fn parent() {
    assert_eq!(
      query("string < object"),
      Expression::Parent {
        child: Box::new(Expression::Kind("string".to_string())),
        parent: Box::new(Expression::Kind("object".to_string()))
      }
    );
  }

  #[test]
  fn descendant() {
    assert_eq!(
      query("object string"),
      Expression::Descendant {
        ancestor: Box::new(Expression::Kind("object".to_string())),
        descendant: Box::new(Expression::Kind("string".to_string()))
      }
    );
  }

  #[test]
  fn direct_child() {
    assert_eq!(
      query("^string"),
      Expression::DirectChild(Box::new(Expression::Kind("string".to_string())))
    );
  }

  #[test]
  fn index() {
    assert_eq!(
      query("string[2]"),
      Expression::Index {
        index: 2,
        query: Box::new(Expression::Kind("string".to_string())),
      }
    );
  }

  #[test]
  fn index_grouped_query() {
    assert_eq!(
      query("(string, number)[1]"),
      Expression::Index {
        index: 1,
        query: Box::new(Expression::Union(vec![
          Expression::Kind("string".to_string()),
          Expression::Kind("number".to_string()),
        ])),
      }
    );
  }

  #[test]
  fn index_hierarchical_query() {
    assert_eq!(
      query("(object > string)[0]"),
      Expression::Index {
        index: 0,
        query: Box::new(Expression::Child {
          child: Box::new(Expression::Kind("string".to_string())),
          parent: Box::new(Expression::Kind("object".to_string())),
        }),
      }
    );
  }

  #[test]
  fn kind() {
    assert_eq!(query("string"), Expression::Kind("string".to_string()));
  }

  #[test]
  fn position() {
    assert_eq!(query("@1"), Expression::Position(1));
  }

  #[test]
  fn union() {
    assert_eq!(
      query("string, number"),
      Expression::Union(vec![
        Expression::Kind("string".to_string()),
        Expression::Kind("number".to_string())
      ])
    );
  }

  #[test]
  fn union_query_with_descendant_and_direct_child() {
    assert_eq!(
      query("object > array[0] string @ 2, ^number > boolean"),
      Expression::Union(vec![
        Expression::Descendant {
          ancestor: Box::new(Expression::Descendant {
            ancestor: Box::new(Expression::Child {
              parent: Box::new(Expression::Kind("object".to_string())),
              child: Box::new(Expression::Index {
                index: 0,
                query: Box::new(Expression::Kind("array".to_string())),
              }),
            }),
            descendant: Box::new(Expression::Kind("string".to_string())),
          }),
          descendant: Box::new(Expression::Position(2)),
        },
        Expression::Child {
          parent: Box::new(Expression::DirectChild(Box::new(
            Expression::Kind("number".to_string())
          ))),
          child: Box::new(Expression::Kind("boolean".to_string())),
        },
      ])
    );
  }

  #[test]
  fn child_with_implicit_current() {
    assert_eq!(
      query("> string"),
      Expression::Child {
        parent: Box::new(Expression::Current),
        child: Box::new(Expression::Kind("string".to_string()))
      }
    );
  }

  #[test]
  fn parent_with_implicit_current() {
    assert_eq!(
      query("< object"),
      Expression::Parent {
        child: Box::new(Expression::Current),
        parent: Box::new(Expression::Kind("object".to_string()))
      }
    );
  }

  #[test]
  fn at_without_number() {
    assert_matches!(
      Parser::parse(Lexer::lex("@").unwrap()),
      Err(Error::UnexpectedEnd)
    );
  }

  #[test]
  fn at_with_invalid_token() {
    assert_matches!(
      Parser::parse(Lexer::lex("@ string").unwrap()),
      Err(Error::UnexpectedToken)
    );
  }

  #[test]
  fn missing_closing_bracket() {
    assert_matches!(
      Parser::parse(Lexer::lex("string[1").unwrap()),
      Err(Error::UnexpectedEnd)
    );
  }

  #[test]
  fn missing_closing_parenthesis() {
    assert_matches!(
      Parser::parse(Lexer::lex("(string").unwrap()),
      Err(Error::UnexpectedEnd)
    );
  }

  #[test]
  fn missing_index() {
    assert_matches!(
      Parser::parse(Lexer::lex("string[").unwrap()),
      Err(Error::UnexpectedEnd)
    );
  }

  #[test]
  fn invalid_index_with_bracket() {
    assert_matches!(
      Parser::parse(Lexer::lex("string[]").unwrap()),
      Err(Error::InvalidIndex)
    );
  }

  #[test]
  fn invalid_index_with_string() {
    assert_matches!(
      Parser::parse(Lexer::lex("string[abc]").unwrap()),
      Err(Error::InvalidIndex)
    );
  }

  #[test]
  fn unexpected_token_start() {
    assert_matches!(
      Parser::parse(Lexer::lex("[").unwrap()),
      Err(Error::UnexpectedToken)
    );
  }

  #[test]
  fn unexpected_token_comma() {
    assert_matches!(
      Parser::parse(Lexer::lex(",").unwrap()),
      Err(Error::UnexpectedToken)
    );
  }

  #[test]
  fn unexpected_trailing_parenthesis() {
    assert_matches!(
      Parser::parse(Lexer::lex("string)").unwrap()),
      Err(Error::UnexpectedToken)
    );
  }

  #[test]
  fn unexpected_trailing_token() {
    assert_matches!(
      Parser::parse(Lexer::lex("string]").unwrap()),
      Err(Error::UnexpectedToken)
    );
  }
}
