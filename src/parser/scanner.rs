use anyhow::{bail, Result};

use crate::parser::token::{Token, TokenType};

pub struct Scanner {
    source: String,

    tokens: Vec<Token>,

    line: u32,
    column: u32,
    current_index: usize,
    start_index: usize,
}

impl From<String> for Scanner {
    fn from(value: String) -> Self {
        let mut scan = Scanner::new();

        scan.source = value;

        scan
    }
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 0,
            tokens: vec![],
            start_index: 0,
            current_index: 0,
            source: String::new(),
        }
    }

    pub fn scan(&mut self, input: &String) -> Result<()> {
        self.source = input.clone();

        while !self.at_end() {
            // We are at the beginning of the next lexeme.
            self.start_index = self.current_index;

            self.scan_tokens()?;
        }

        self.tokens.push(Token::new(TokenType::EOF, "\0".to_string(), self.line as u64, self.column as u64));

        Ok(())
    }

    fn scan_tokens(&mut self) -> Result<()> {
        let c = self.advance();

        match c {
            "(" => {
                self.add_token(TokenType::LeftParen);
            }

            ")" => {
                self.add_token(TokenType::RightParen);
            }

            "{" => {
                self.add_token(TokenType::LeftBrace);
            }

            "}" => {
                self.add_token(TokenType::RightBrace);
            }

            "," => {
                self.add_token(TokenType::COMMA);
            }

            "." => {
                self.add_token(TokenType::DOT);
            }

            "-" => {
                self.add_token(TokenType::MINUS);
            }

            "+" => {
                self.add_token(TokenType::PLUS);
            }

            ";" => {
                self.add_token(TokenType::SEMICOLON);
            }

            "*" => {
                self.add_token(TokenType::STAR);
            }

            // for these s (!, =, <, >) it can be a single char, or it can be followed by = (!=, ==, <=, >=)
            "!" => {
                let token_type = if self.match_char("=") { TokenType::BangEqual } else { TokenType::BANG };

                self.add_token(token_type);
            }

            "=" => {
                self.add_token(TokenType::EQUAL);
            }

            "<" => {
                let token_type = if self.match_char("=") { TokenType::LessEqual } else { TokenType::LESS };

                self.add_token(token_type);
            }

            ">" => {
                let token_type = if self.match_char("=") { TokenType::GreaterEqual } else { TokenType::GREATER };

                self.add_token(token_type);
            }

            "/" => {
                self.add_token(TokenType::SLASH);
            }

            "#" => {
                // A comment goes until the end of the line.
                while self.peek() != "\n" && !self.at_end() {
                    self.advance();
                }
            }

            "\"" => {
                self.parse_string();
            }

            " " |
            "\r" |
            "\t" => {

                // Ignore whitespace.
            }

            "\n" => {
                self.increment_line();
            }

            _ => {
                if Scanner::is_digit(c) {
                    self.parse_number();
                } else if Scanner::is_alpha(c) {
                    self.parse_identifier();
                } else {
                    bail!("SyntaxError: Unexpected character. at {}:{}", self.line, self.column)
                }
            }
        };

        Ok(())
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn at_end(&self) -> bool {
        self.current_index == self.source.len()
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start_index..self.current_index];

        self.tokens.push(Token::new(token_type, text.to_string(), self.line as u64, self.column as u64));
    }

    fn increment_line(&mut self) {
        self.column = 1;

        self.line += 1;
    }

    fn match_char(&mut self, expected: &str) -> bool {
        if self.at_end() { return false; };

        if &self.source[self.current_index..self.current_index + 1] != expected { return false; };

        self.column += 1;

        self.current_index += 1;

        true
    }

    fn peek(&self) -> &str {
        if self.at_end() { return "\0"; };

        &self.source[self.current_index..self.current_index + 1]
    }

    fn peek_next(&self) -> &str {
        if self.current_index + 1 >= self.source.len() { return "\0"; };

        &self.source[self.current_index + 1..self.current_index + 2]
    }

    fn advance(&mut self) -> &str {
        let character = &self.source[self.current_index..self.current_index + 1];

        self.column += 1;

        self.current_index += 1;

        character
    }
    
    fn parse_string(&mut self) {
        while self.peek() != "\"" && !self.at_end() {
            if self.peek() == "\n" { self.increment_line() };

            self.advance();
        }

        if self.at_end() {
            return;
        }

        // The closing ".
        self.advance();

        self.add_token(TokenType::STRING);
    }

    fn parse_number(&mut self) {
        while Scanner::is_digit(self.peek()) { self.advance(); };

        // Look for a fractional part.
        if self.peek() == "." && Scanner::is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();

            //noinspection WhileCanBeDoWhile
            while Scanner::is_digit(self.peek()) { self.advance(); };
        }

        self.add_token(TokenType::NUMBER);
    }

    pub fn is_digit(st: &str) -> bool {
        st.chars().all(|c| c >= '0' && c <= '9')
    }

    fn parse_identifier(&mut self) {
        while Scanner::is_alpha_numeric(self.peek()) { self.advance(); };

        let text = &self.source[self.start_index..self.current_index];

        let token_type: TokenType = TokenType::from(text);

        self.add_token(token_type);
    }

    pub fn is_alpha(st: &str) -> bool {
        st.chars().all(|c| (c >= 'a' && c <= 'z') ||
            (c >= 'A' && c <= 'Z') ||
            c == '_')
    }

    pub fn is_alpha_numeric(c: &str) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }
}