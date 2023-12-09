use crate::tokenizer::cursor::Cursor;
use crate::tokenizer::token_type::TokenType;

pub mod token_type;
pub mod cursor;

pub struct Token {
    pub kind: TokenType,
    pub len: u32,
}

impl Token {
    fn new(kind: TokenType, len: u32) -> Token {
        Token { kind, len }
    }
}

impl Cursor<'_> {
    pub fn next_token(&mut self) -> Token {
        Token::new(TokenType::Eq, 123)
    }
}
