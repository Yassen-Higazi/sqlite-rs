use crate::parser::token::{Token, TokenType};
use anyhow::{anyhow, bail, Result};

pub enum StatementType {
    INSERT,
    SELECT,
    UPDATE,
    DELETE,
    ALTER,
}

impl From<TokenType> for StatementType {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::SELECT => StatementType::SELECT,
            TokenType::INSERT => StatementType::INSERT,
            TokenType::UPDATE => StatementType::UPDATE,
            TokenType::DELETE => StatementType::DELETE,
            TokenType::ALTER => StatementType::ALTER,
            _ => panic!("{}", anyhow!("Invalid Statement Type"))
        }
    }
}

pub struct Statement {
    pub limit: Option<u64>,
    pub order: Vec<Token>,
    pub tables: Vec<Token>,
    pub columns: Vec<Token>,
    pub on_conditions: Vec<Token>,
    pub where_conditions: Vec<Token>,
    pub having_conditions: Vec<Token>,
    pub statement_type: StatementType,
}

impl Statement {
    pub fn new(tokens: &Vec<Token>) -> Result<Self> {
        let allowed_token_types_in_where = vec![TokenType::IDENTIFIER, TokenType::EQUAL, TokenType::BangEqual, TokenType::GREATER, TokenType::GreaterEqual, TokenType::LESS, TokenType::LessEqual, TokenType::BETWEEN, TokenType::AND, TokenType::OR, TokenType::NUMBER, TokenType::STRING];

        let mut index: usize = 0;

        let mut statement = Self {
            limit: None,
            order: vec![],
            tables: vec![],
            columns: vec![],
            on_conditions: vec![],
            where_conditions: vec![],
            having_conditions: vec![],
            statement_type: StatementType::INSERT,
        };

        while index < tokens.len() {
            let token = &tokens[index];

            match token.token_type {
                TokenType::INSERT => {
                    statement.statement_type = StatementType::INSERT;
                }

                TokenType::SELECT => {
                    statement.statement_type = StatementType::SELECT;

                    let mut next_index = index + 1;

                    if next_index >= tokens.len() {
                        bail!("Select statement must be followed by an Identifier or *, line: {}:{}", token.line, token.column);
                    }

                    let next_token = &tokens[next_index];

                    if next_token.token_type != TokenType::IDENTIFIER || next_token.token_type == TokenType::STAR {
                        bail!("Select statement must be followed by an Identifier or *, line: {}:{}", next_token.line, next_token.column);
                    }

                    statement.columns.push(next_token.clone());

                    next_index += 1;

                    let new_token = &tokens[next_index];

                    if next_token.token_type == TokenType::STAR && new_token.token_type == TokenType::IDENTIFIER {
                        bail!("Syntax Error at line {}:{}", new_token.line, new_token.column)
                    }

                    loop {
                        let next_token = &tokens[next_index];

                        if next_token.token_type == TokenType::IDENTIFIER {
                            statement.columns.push(next_token.clone());

                            next_index += 1;
                        } else if next_token.token_type == TokenType::COMMA {
                            next_index += 1;
                        } else {
                            break;
                        }
                    }

                    index = next_index;
                }

                TokenType::UPDATE => {
                    statement.statement_type = StatementType::UPDATE;
                }

                TokenType::DELETE => {
                    statement.statement_type = StatementType::DELETE;
                }

                TokenType::ALTER => {
                    statement.statement_type = StatementType::ALTER;
                }

                TokenType::FROM => {
                    let mut next_index = index + 1;

                    if next_index >= tokens.len() {
                        bail!("From statement must be followed by an Identifier, line: {}:{}", token.line, token.column);
                    }

                    let next_token = &tokens[next_index];

                    if next_token.token_type != TokenType::IDENTIFIER {
                        bail!("From statement must be followed by an Identifier, line: {}:{}", next_token.line, next_token.column);
                    }

                    statement.tables.push(next_token.clone());

                    next_index += 1;

                    loop {
                        let next_token = &tokens[next_index];

                        if next_token.token_type == TokenType::IDENTIFIER {
                            statement.tables.push(next_token.clone());

                            next_index += 1;
                        } else {
                            break;
                        }
                    }

                    index = next_index;
                }

                TokenType::WHERE => {
                    let mut next_index = index + 1;

                    if next_index >= tokens.len() {
                        bail!("Where statement must be followed by an Identifier, line: {}:{}", token.line, token.column);
                    }

                    let next_token = &tokens[next_index];

                    if next_token.token_type != TokenType::IDENTIFIER {
                        bail!("Where statement must be followed by an Identifier, line: {}:{}", next_token.line, next_token.column);
                    }

                    statement.where_conditions.push(next_token.clone());

                    next_index += 1;

                    loop {
                        let next_token = &tokens[next_index];

                        if allowed_token_types_in_where.contains(&next_token.token_type) {
                            statement.where_conditions.push(next_token.clone());

                            next_index += 1;
                        } else {
                            break;
                        }
                    }

                    index = next_index;
                }

                TokenType::ORDER => {}

                TokenType::BY => {}

                TokenType::AND => {}

                TokenType::OR => {}

                TokenType::LIMIT => {
                    let next_index = index + 1;

                    if next_index >= tokens.len() {
                        bail!("LIMIT statement must be followed by a number");
                    }

                    let limit_token = &tokens[next_index];

                    if limit_token.token_type != TokenType::NUMBER {
                        bail!("LIMIT statement must be followed by a number, line: {}:{}", limit_token.line, limit_token.column);
                    }

                    let limit_value = limit_token.lexeme.parse::<u64>()?;

                    statement.limit = Some(limit_value);

                    index = next_index + 1;
                }

                TokenType::EOF => {
                    break;
                }

                TokenType::LeftParen => {
                    index += 1;
                }

                TokenType::RightParen => {
                    index += 1;
                }

                TokenType::STAR => {
                    index += 1;
                }

                TokenType::DOT => {
                    index += 1;
                }

                TokenType::IDENTIFIER => {
                    index += 1;
                }

                _ => bail!("Invalid Token {:?}", token)
            }
        }

        Ok(statement)
    }
}