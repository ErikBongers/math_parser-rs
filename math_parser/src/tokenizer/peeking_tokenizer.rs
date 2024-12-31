use crate::errors::Error;
use crate::globals::SourceIndex;
use crate::globals::sources::Source;
use crate::number::Number;
use crate::tokenizer::cursor::Cursor;
use crate::tokenizer::Token;

#[derive(Clone)]
pub struct PeekingTokenizer<'a> {
    cur: Cursor<'a>,
    prev_cur: Cursor<'a>,
    peeked_token: Token,
    current_number: Number,
}

impl<'a> PeekingTokenizer<'a> {
    pub fn new(source: &'a Source) -> Self {
        let mut cur =  Cursor::new(source);
        let prev_cur = cur.clone();
        let current_number = cur.number.clone(); //before setting peeked_token!
        let peeked_token = cur.next_token();
        PeekingTokenizer {
            cur,
            prev_cur,
            peeked_token,
            current_number
        }
    }

    pub fn get_errors(&self) -> &Vec<Error> {self.cur.errors.as_ref()}
    #[inline]
    pub fn source_index(&self) -> SourceIndex { self.cur.source_index() }

    pub fn peek(&self) -> &Token {
        &self.peeked_token
    }

    pub fn peek_second(&mut self) -> Token {
        //store state.
        let old_cur = self.cur.clone();
        let old_token = self.peeked_token.clone();
        let old_number = self.current_number.clone();

        self.next();
        let t = self.peeked_token.clone();

        //restore state
        self.cur = old_cur;
        self.peeked_token = old_token;
        self.current_number = old_number;

        t
    }

    pub fn next(&mut self)  -> Token {
        self.prev_cur = self.cur.clone();
        let t = self.peeked_token.clone();
        self.current_number = self.cur.number.clone(); //before setting peeked_token!
        self.peeked_token = self.cur.next_token();
        return t;
    }

    pub fn set_nl_is_token(&mut self, is_token: bool) {
        if self.cur.nl_is_token == is_token { return; }

        self.cur = self.prev_cur.clone();
        self.cur.nl_is_token = is_token;
        self.peeked_token = self.cur.next_token();
    }

    pub fn set_dot_and_comma_decimal(&mut self, is_decimal: bool) {
        if self.cur.is_dot_and_comma_decimal == is_decimal { return; }

        self.cur = self.prev_cur.clone();
        self.cur.is_dot_and_comma_decimal = is_decimal;
        self.peeked_token = self.cur.next_token();
    }

    /// Gets the last numeric value that has been found **_past_** the last next() or _**at**_ the current peek()
    pub fn get_number(&self) -> Number {
        self.current_number.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::globals::Globals;
    use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;

    #[test]
    fn test_peeking() {
        let mut globals = Globals::new();
        let src_name = "src1";
        let text = "111 + 222 = abbc;\n!//comment";
        globals.set_source(src_name.to_string(), text.to_string());
        let mut tok = PeekingTokenizer::new(globals.get_source_by_name(src_name).unwrap()); //unwrap ok: we just pushed a source.
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
        assert_eq!(n3.significand, 111.0);
        let t3 = tok.next();
        let n3 = tok.get_number();
        assert_eq!(n3.significand, 222.0);
        assert_ne!(t1.kind, t2.kind);
        assert_ne!(t2.kind, t3.kind);

        assert_eq!(peek2.kind, t2.kind);
    }
}