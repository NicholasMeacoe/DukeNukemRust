#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    Number(i32),
    Str(String),
    LeftBrace,
    RightBrace,
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.pos < self.chars.len() {
            self.skip_whitespace();

            if self.pos >= self.chars.len() {
                break;
            }

            let ch = self.chars[self.pos];

            // Single line comment: //
            if ch == '/' && self.peek(1) == Some('/') {
                self.pos += 2;
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            }

            // Multi-line comment: /* ... */
            if ch == '/' && self.peek(1) == Some('*') {
                self.pos += 2;
                while self.pos < self.chars.len() {
                    if self.chars[self.pos] == '*' && self.peek(1) == Some('/') {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
                continue;
            }

            if ch == '{' {
                tokens.push(Token::LeftBrace);
                self.pos += 1;
                continue;
            }

            if ch == '}' {
                tokens.push(Token::RightBrace);
                self.pos += 1;
                continue;
            }

            // String literal: "..."
            if ch == '"' {
                self.pos += 1;
                let mut s = String::new();
                while self.pos < self.chars.len() && self.chars[self.pos] != '"' {
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                if self.pos < self.chars.len() && self.chars[self.pos] == '"' {
                    self.pos += 1;
                }
                tokens.push(Token::Str(s));
                continue;
            }

            // Number or Identifier / keyword / formatted string (01:45, E1L1.map, etc.)
            if ch.is_ascii_digit()
                || is_ident_char(ch)
                || (ch == '-' && self.peek(1).map_or(false, |c| c.is_ascii_digit()))
            {
                let start = self.pos;
                if ch == '-' {
                    self.pos += 1;
                }
                while self.pos < self.chars.len() && is_ident_char(self.chars[self.pos]) {
                    self.pos += 1;
                }
                let token_str: String = self.chars[start..self.pos].iter().collect();
                if let Ok(num) = token_str.parse::<i32>() {
                    if !token_str.contains(':') && !token_str.contains('.') {
                        tokens.push(Token::Number(num));
                    } else {
                        tokens.push(Token::Ident(token_str));
                    }
                } else {
                    tokens.push(Token::Ident(token_str));
                }
                continue;
            }

            // Unknown character, skip
            self.pos += 1;
        }

        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_whitespace() || ch == '\r' || ch == '\n' || ch == '\t' || ch == ',' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }
}

fn is_ident_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == '$' || ch == ':'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let script = r#"
            define PISTOL_STRENGTH 6
            // This is a comment
            action ATROOPWALK 0 4 5 1 12
            /* Multi-line
               comment here */
            actor 1680 100 {
                ifpdistl 1024
                    sound 15
            } enda
        "#;

        let mut lexer = Lexer::new(script);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("define".into()));
        assert_eq!(tokens[1], Token::Ident("PISTOL_STRENGTH".into()));
        assert_eq!(tokens[2], Token::Number(6));
        assert_eq!(tokens[3], Token::Ident("action".into()));
        assert_eq!(tokens[4], Token::Ident("ATROOPWALK".into()));
        assert_eq!(tokens[5], Token::Number(0));
        assert_eq!(tokens[6], Token::Number(4));
        assert_eq!(tokens[7], Token::Number(5));
        assert_eq!(tokens[8], Token::Number(1));
        assert_eq!(tokens[9], Token::Number(12));
        assert_eq!(tokens[10], Token::Ident("actor".into()));
        assert_eq!(tokens[11], Token::Number(1680));
        assert_eq!(tokens[12], Token::Number(100));
        assert_eq!(tokens[13], Token::LeftBrace);
        assert_eq!(tokens[14], Token::Ident("ifpdistl".into()));
        assert_eq!(tokens[15], Token::Number(1024));
        assert_eq!(tokens[16], Token::Ident("sound".into()));
        assert_eq!(tokens[17], Token::Number(15));
        assert_eq!(tokens[18], Token::RightBrace);
        assert_eq!(tokens[19], Token::Ident("enda".into()));
    }

    #[test]
    fn test_lexer_negative_numbers_and_strings() {
        let script = r#"define NEG -45 definelevelname 0 0 "E1L1.MAP" 100 200 "HOLLYWOOD""#;
        let mut lexer = Lexer::new(script);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("define".into()));
        assert_eq!(tokens[1], Token::Ident("NEG".into()));
        assert_eq!(tokens[2], Token::Number(-45));
        assert_eq!(tokens[3], Token::Ident("definelevelname".into()));
        assert_eq!(tokens[4], Token::Number(0));
        assert_eq!(tokens[5], Token::Number(0));
        assert_eq!(tokens[6], Token::Str("E1L1.MAP".into()));
        assert_eq!(tokens[7], Token::Number(100));
        assert_eq!(tokens[8], Token::Number(200));
        assert_eq!(tokens[9], Token::Str("HOLLYWOOD".into()));
    }
}
