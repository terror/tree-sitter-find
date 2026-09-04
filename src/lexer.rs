use {
  super::*,
  std::{iter::Peekable, str::CharIndices},
};

#[derive(Debug)]
pub(crate) struct Lexer<'a> {
  characters: Peekable<CharIndices<'a>>,
  input_length: usize,
}

impl<'a> Lexer<'a> {
  pub(crate) fn new(input: &'a str) -> Self {
    Self {
      characters: input.char_indices().peekable(),
      input_length: input.len(),
    }
  }

  fn advance(&mut self) -> Option<char> {
    self.characters.next().map(|(_, character)| character)
  }

  fn character(&mut self) -> Option<char> {
    self.characters.peek().map(|(_, character)| *character)
  }

  fn position(&mut self) -> usize {
    self
      .characters
      .peek()
      .map_or(self.input_length, |(position, _)| *position)
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
          '(' => {
            lexer.advance();
            tokens.push(Token::LeftParen);
          }
          ']' => {
            lexer.advance();
            tokens.push(Token::RightBracket);
          }
          ')' => {
            lexer.advance();
            tokens.push(Token::RightParen);
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
            return Err(Error::UnexpectedCharacter {
              character,
              position: lexer.position(),
            });
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
      Lexer::lex("(recipe[0] > identifier)").unwrap(),
      vec![
        Token::LeftParen,
        Token::Kind("recipe".to_string()),
        Token::LeftBracket,
        Token::Number(0),
        Token::RightBracket,
        Token::Greater,
        Token::Kind("identifier".to_string()),
        Token::RightParen,
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

  #[test]
  fn unexpected_character() {
    assert_matches!(
      Lexer::lex("recipe!").unwrap_err(),
      Error::UnexpectedCharacter {
        character: '!',
        position: 6,
      }
    );
  }

  #[test]
  fn unexpected_character_position_is_a_byte_offset() {
    assert_matches!(
      Lexer::lex("é!").unwrap_err(),
      Error::UnexpectedCharacter {
        character: '!',
        position: 2,
      }
    );
  }
}
