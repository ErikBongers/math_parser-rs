use std::str::Chars;

pub struct Cursor<'a> {
    chars: Chars<'a>,
}

pub(crate) const EOF_CHAR: char = '\0';

// https://doc.rust-lang.org/beta/nightly-rustc/rustc_lexer/index.html
impl<'a> Cursor<'a> {
    pub fn new (text: &str) -> Cursor {
        Cursor {
            chars: text.chars()
        }
    }

    pub fn peek(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    pub fn peek_second(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    pub fn next(&mut self) -> char {
        self.chars.next().unwrap_or(EOF_CHAR)
    }
}