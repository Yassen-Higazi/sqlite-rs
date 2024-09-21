#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    INSERT,
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
    INTO,
    VALUES,
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
    LIMIT,
    TABLE,
    SET,

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

#[derive(Debug, Clone)]
pub struct Token {
    pub lexeme: String,
    pub token_type: TokenType,
    pub line: u64,
    pub column: u64,
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
            "LIMIT" => TokenType::LIMIT,
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