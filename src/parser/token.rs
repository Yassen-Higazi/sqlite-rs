#[derive(Debug)]
pub enum TokenType {
    SELECT,
    UPDATE,
    DELETE,
    ALTER,
    FROM,
    WHERE,
    ORDER,
    GROUP,
    BY,
    HAVING,
    JOIN,
    NATURAL,
    INNER,
    OUTER,
    FULL,
    ON,
    AND,
    OR,
    BETWEEN,
    AS,
    STRING,
    NUMBER,
    IDENTIFIER,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    STAR,
    BangEqual,
    BANG,
    EQUAL,
    LessEqual,
    LESS,
    GreaterEqual,
    GREATER,
    SLASH,
    HASH,

    COUNT,
    EOF,
}

#[derive(Debug)]
pub struct Token {
    lexeme: String,
    token_type: TokenType,
    line: u64,
    column: u64,
}

impl From<&str> for TokenType {
    fn from(value: &str) -> Self {
        TokenType::from(value.to_string())
    }
}

impl From<String> for TokenType {
    fn from(value: String) -> Self {
        match value.to_uppercase().as_str() {
            "SELECT" => TokenType::SELECT,
            "UPDATE" => TokenType::UPDATE,
            "DELETE" => TokenType::DELETE,
            "ALTER" => TokenType::ALTER,
            "FROM" => TokenType::FROM,
            "WHERE" => TokenType::WHERE,
            "ORDER" => TokenType::ORDER,
            "GROUP" => TokenType::GROUP,
            "BY" => TokenType::BY,
            "HAVING" => TokenType::HAVING,
            "JOIN" => TokenType::JOIN,
            "NATURAL" => TokenType::NATURAL,
            "INNER" => TokenType::INNER,
            "OUTER" => TokenType::OUTER,
            "FULL" => TokenType::FULL,
            "ON" => TokenType::ON,
            "AND" => TokenType::AND,
            "OR" => TokenType::OR,
            "BETWEEN" => TokenType::BETWEEN,
            "AS" => TokenType::AS,
            "STRING" => TokenType::STRING,
            "NUMBER" => TokenType::NUMBER,
            _ => TokenType::IDENTIFIER
        }
    }
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: u64, column: u64) -> Self {
        Self {
            line,
            column,
            lexeme,
            token_type,
        }
    }
}