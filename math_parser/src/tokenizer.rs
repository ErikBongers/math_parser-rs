use crate::globals::SourceIndex;
use crate::tokenizer::cursor::{Cursor, is_id_continue, is_id_start, Range};
use crate::tokenizer::token_type::TokenType;
use crate::tokenizer::token_type::TokenType::*;

pub mod token_type;
pub mod cursor;
pub mod peeking_tokenizer;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub range :Range,
    #[cfg(debug_assertions)]
    pub text: String,
}
impl Token {
    fn new(kind: TokenType, source_index :SourceIndex, start: usize, end :usize, _text: String) -> Token {
        Token { kind, range: Range { source_index, start, end },
            #[cfg(debug_assertions)]
            text: _text
        }
    }
}

impl Cursor<'_> {
    #[inline]
    pub fn source_index(&self) -> SourceIndex { self.source.index }

    pub fn next_token(&mut self) -> Token {
        self.eat_whitespace();
        let mut start_pos = self.get_pos();
        self.is_beginning_of_text = false; // clear this once per next_token instead of once per next(), for performance.
        let first_char = match self.next() {
            None => return Token::new(Eot, self.source.index, 0, 0, "".to_string()),
            Some(c) => c
        };
        let token_type = match first_char {
            '\n' => Newline, //only if nl_is_token == true
            '{' => CurlOpen,
            '}' => CurlClose,
            '(' => ParOpen,
            ')' => ParClose,
            '[' => BracOpen,
            ']' => BracClose,
            '^' => Power,
            '=' => Eq,
            ',' => Comma,
            '|' => Pipe,
            ';' => SemiColon,
            '%' => {
                if self.peek() == '%' {
                    self.next(); //eat
                    Modulo
                } else {
                    Percent
                }
            },
            '!' => {
                match (self.peek(), self.peek_second()) {
                    ('/', '/') => {
                        self.next();
                        self.next();
                        self.get_to_eol();
                        start_pos += 3; //remove the "!//" from the string.
                        EchoCommentLine
                    },
                    _ => Exclam
                }
            },
            '.' => {
              match (self.peek(), self.peek_second()) {
                  ('.', '.') => {
                      self.next();
                      self.next();
                      Ellipsis
                  },
                  ('=', _) => {
                      self.next();
                      EqUnit
                  },
                  ('0'..='9', _) => {
                    self.number = self.parse_number('.');
                      Number
                  },
                  _ => Dot
              }
            },

            '+' => {
                match self.peek() {
                    '=' => { self.next(); EqPlus },
                    '+' => { self.next(); Inc },
                    _ => Plus
                }
            },
            '-' => {
                match self.peek() {
                    '=' => { self.next(); EqMin },
                    '-' => { self.next(); Dec },
                    _ => Min
                }
            },
            '*' => {
                if self.peek() == '=' {
                    self.next();
                    EqMult
                } else {
                    Mult
                }
            },
            '#' => {
                if self.match_word("define") {
                    Define
                } else {
                    if self.match_word("undef") {
                        Undef
                    } else {
                        if self.peek() == '/' {
                            self.next();
                            MuteEnd
                        } else {
                            MuteLine
                        }
                    }
                }
            },
            '/' => {
                match self.peek() {
                    '=' => {
                        self.next();
                        EqDiv
                    },
                    '/' => {
                        self.get_to_eol();
                        // CommentLine
                        return self.next_token();
                    },
                    '*' => {
                        self.get_to_end_of_comment();
                        return self.next_token();
                    },
                    '#' => {
                        self.next();
                        MuteStart
                    },
                    _ => Div
                }
            },
            '\'' => {
                self.eat_while(
                     |c| if let '\'' = c {
                        false
                    } else {
                        true
                    }
                );
                self.next(); //eat end quote
                QuotedStr
            },
            c @ ('0'..='9') => {
                self.number = self.parse_number(c);
                Number
            },
            c if is_id_start(c) => {
                self.eat_while(is_id_continue);
                let id = &self.source.get_text()[start_pos..self.get_pos()];
                match id {
                    "function" => Function,
                    _ => Id
                }
            },

            _ => Unknown
        };
        let res = Token::new(token_type, self.source.index, start_pos, self.get_pos(), self.source.get_text()[start_pos..self.get_pos()].to_string());
        res
    }

    fn match_word(&mut self, word: &str) -> bool{
        let mut chars2 = self.chars.clone();
        if chars2.as_str().starts_with(word) {
            chars2.nth(word.len()-1);
            //next should not be an id character.
            if let Some(c) = chars2.next() {
                if is_id_continue(c) == false {
                    self.chars.nth(word.len()-1);
                    return true
                }
            }
        }
        false
    }
}
