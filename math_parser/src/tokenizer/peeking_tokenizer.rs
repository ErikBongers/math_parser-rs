use crate::globals::sources::Source;
use crate::tokenizer::cursor::{Cursor, Number};
use crate::tokenizer::Token;

#[derive(Clone)]
pub struct PeekingTokenizer<'a> {
    cur: Cursor<'a>,
    prev_cur: Cursor<'a>,
    peeked_token: Token,
}

impl<'a> PeekingTokenizer<'a> {
    pub fn new(source: &'a Source) -> Self {
        let mut cur =  Cursor::new(source);
        let prev_cur = cur.clone();
        let peeked_token = cur.next_token();
        PeekingTokenizer {
            cur,
            prev_cur,
            peeked_token,
        }
    }

    #[inline]
    pub fn source_index(&self) -> u8 { self.cur.source_index() as u8 }

    pub fn peek(&self) -> &Token {
        &self.peeked_token
    }

    pub fn peek_second(&mut self) -> Token {
        //store state.
        let old_cur = self.cur.clone();
        let old_token = self.peeked_token.clone();

        self.next();
        let t = self.peeked_token.clone();

        //restore state
        self.cur = old_cur;
        self.peeked_token = old_token;

        t
    }
    pub fn next(&mut self)  -> Token {
        self.prev_cur = self.cur.clone();
        let t = self.peeked_token.clone();
        self.peeked_token = self.cur.next_token();
        return t;
    }

    pub fn set_ln_is_token(&mut self, is_token: bool) {
        if(self.cur.ln_is_token == is_token) { return; }

        self.cur = self.prev_cur.clone();
        self.cur.ln_is_token = is_token;
        self.peeked_token = self.cur.next_token();
    }

    pub fn get_number(&self) -> Number {
        self.cur.number.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;

    #[test]
    fn test_peeking() {
        let mut tok = PeekingTokenizer::new("111 + 222 = abbc;\n!//comment");
        //start with peeking '+'
        let peek2 = tok.peek_second();
        // peek 111
        let t1 = tok.peek().clone();
        assert_eq!(t1.kind, tok.next().kind);
        let n1 = tok.get_number();
        assert_eq!(n1.significand, 111.0);
        //peek '+'
        let t2 = tok.peek().clone();
        assert_eq!(t2.kind, tok.next().kind);
        //peek 333
        let t3 = tok.peek();
        let n3 = tok.get_number();
        assert_eq!(n3.significand, 222.0);
        assert_ne!(t1.kind, t2.kind);
        assert_ne!(t2.kind, t3.kind);

        assert_eq!(peek2.kind, t2.kind);
    }
}