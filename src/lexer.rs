use super::*;

#[derive(Debug)]
pub(crate) struct Lexer<'a> {
  input: &'a str,
  position: usize,
}

impl<'a> Lexer<'a> {
  pub(crate) fn new(input: &'a str) -> Self {
    Self { input, position: 0 }
  }

  fn advance(&mut self) -> Option<char> {
    let character = self.character();
    self.position += 1;
    character
  }

  fn character(&self) -> Option<char> {
    self.input.chars().nth(self.position)
  }

  pub(crate) fn lex(input: &'a str) -> Result<Vec<Token>> {
    let mut lexer = Self::new(input);

    let mut tokens = Vec::new();

    while lexer.character().is_some() {
      lexer.skip_whitespace();

      if let Some(character) = lexer.character() {
        match character {
          '[' => {
            lexer.advance();
            tokens.push(Token::LeftBracket);
          }
          ']' => {
            lexer.advance();
            tokens.push(Token::RightBracket);
          }
          ',' => {
            lexer.advance();
            tokens.push(Token::Comma);
          }
          '>' => {
            lexer.advance();
            tokens.push(Token::Greater);
          }
          '<' => {
            lexer.advance();
            tokens.push(Token::Less);
          }
          '^' => {
            lexer.advance();
            tokens.push(Token::Caret);
          }
          '@' => {
            lexer.advance();
            tokens.push(Token::At);
          }
          character if character.is_ascii_digit() => {
            tokens.push(Token::Number(lexer.lex_number()?));
          }
          character if character.is_alphabetic() || character == '_' => {
            tokens.push(Token::Kind(lexer.lex_identifier()));
          }
          _ => {
            lexer.advance();
          }
        }
      }
    }

    Ok(tokens)
  }

  fn lex_identifier(&mut self) -> String {
    let mut result = String::new();

    while let Some(character) = self.character() {
      if character.is_alphanumeric() || character == '_' {
        result.push(character);
        self.advance();
      } else {
        break;
      }
    }

    result
  }

  fn lex_number(&mut self) -> Result<usize> {
    let mut result = String::new();

    while let Some(character) = self.character() {
      if character.is_ascii_digit() {
        result.push(character);
        self.advance();
      } else {
        break;
      }
    }

    result.parse().context(error::ParseNumberSnafu)
  }

  fn skip_whitespace(&mut self) {
    while let Some(character) = self.character() {
      if character.is_whitespace() {
        self.advance();
      } else {
        break;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn basic_tokens() {
    assert_eq!(
      Lexer::lex("recipe[0] > identifier").unwrap(),
      vec![
        Token::Kind("recipe".to_string()),
        Token::LeftBracket,
        Token::Number(0),
        Token::RightBracket,
        Token::Greater,
        Token::Kind("identifier".to_string()),
      ]
    );
  }

  #[test]
  fn parent_token() {
    assert_eq!(
      Lexer::lex("identifier < recipe").unwrap(),
      vec![
        Token::Kind("identifier".to_string()),
        Token::Less,
        Token::Kind("recipe".to_string()),
      ]
    );
  }

  #[test]
  fn invalid_index() {
    assert_matches!(
      Lexer::lex("recipe[18446744073709551616]").unwrap_err(),
      crate::Error::ParseNumber { .. }
    );
  }
}
