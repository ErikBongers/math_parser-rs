use std::str::Chars;
#[derive(Clone)]
pub struct Cursor<'a> {
    pub text: &'a str,
    chars: Chars<'a>,
    len_text: usize,
    pub newline_found: bool,
    pub number: Number,
    pub is_beginning_of_text: bool,
}

#[derive(Debug, Clone)]
pub struct Range {
    pub source_index: u8,
    pub start :usize,
    pub end : usize
}

#[derive(Clone)]
pub struct Number {
    pub significand: f64,
    pub exponent: i32,
    // Unit unit;
    // NumFormat numFormat = NumFormat::DEC;
    // std::vector<Error> errors;
    // Range range;
}

impl Number {
    pub fn new(significand: f64, exponent: i32) -> Self {
        Number {
            significand,
            exponent
        }
    }
}

pub(crate) const EOF_CHAR: char = '\0';

// https://doc.rust-lang.org/beta/nightly-rustc/rustc_lexer/index.html
impl<'a> Cursor<'a> {
    pub fn new (text: &str) -> Cursor {
        Cursor {
            text,
            chars: text.chars(),
            len_text: text.len(),
            newline_found: true, //first line is also a new line!
            number: Number::new(0.0, 0),
            is_beginning_of_text: true,
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

    pub fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn eat_whitespace(&mut self)  {
        self.newline_found = self.is_beginning_of_text; //this will initialy 'reset' the newline_found value to true, since you always start at a newline.
        while self.is_whitespace(self.peek()) {
            let c = self.next();
            if let Some('\n') = c {
                self.newline_found = true;
            }
        }
    }
    pub fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek()) && !self.is_eof() {
            self.next();
        }
    }

    pub fn get_to_end_of_comment(&mut self) {
        loop {
            if let ('*', '/') = (self.peek(), self.peek_second()) {
                self.next();
                self.next();
                break;
            } else {
                if self.is_eof() {
                    break;
                } else {
                    self.next();
                }
            }
        }
    }
    pub fn get_to_eol(&mut self) {
        self.eat_while(
            |c| if let '\n' | '\r' = c
            {
                false
            } else {
                true
            }
        );
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }
    pub fn get_pos(&mut self) -> usize{
        self.len_text - self.chars.as_str().len()
    }

    pub fn is_whitespace(&self, c: char) -> bool {
        match c {
            // Usual ASCII suspects
            '\u{0009}'   // \t
            | '\u{000B}' // vertical tab
            | '\u{000C}' // form feed
            | '\u{000D}' // \r
            | '\u{0020}' // space

            // NEXT LINE from latin1
            | '\u{0085}'

            // Bidi markers
            | '\u{200E}' // LEFT-TO-RIGHT MARK
            | '\u{200F}' // RIGHT-TO-LEFT MARK

            // Dedicated whitespace characters from Unicode
            | '\u{2028}' // LINE SEPARATOR
            | '\u{2029}' => true, // PARAGRAPH SEPARATOR
            '\n' => true,
            _ => false
        }
    }

    fn parse_integer(&mut self) -> i32 {
        let mut i:i32 = 0;
        let factor = match self.peek() {
            '-' => {
                self.next();
                -1
            },
            '+' => {
                self.next();
                1
            },
            _ => 1
        };
        loop {
            match self.peek() {
                c @ '0'..='9' => {
                    self.next();
                    i = i * 10 + (c as i32 - '0' as i32);
                },
                '_' => { self.next(); },
                _ => break
            }
        }
        i * factor
    }


    fn parse_decimal(&mut self, c: char) -> Number {
        let mut d: f64 = 0.0;
        let mut e: i32 = 0;
        let mut decimal_divider: f64 = 1.0;

        if c == '.' {
            decimal_divider = 10.0;
        } else {
            d = (c as i32 - '0' as i32) as f64;
        };

        loop {
            match self.peek() {
                c @ '0'..='9' => {
                    self.next(); //consume
                    if decimal_divider == 1.0 {
                        d = d * 10.0 + (c as i32 - '0' as i32) as f64;
                    }
                    else{
                        d += (c as i32 - '0' as i32) as f64 / decimal_divider;
                        decimal_divider *= 10.0;
                    }
                },
                '.' => {
                    match self.peek_second() {
                        '0'..='9' => {
                            self.next(); //consume DOT
                            decimal_divider = 10.0;
                        },
                        _ => {
                            break;
                        }
                    }
                },
                '_' => {
                    self.next(); //just eat and ignore it
                },
                _ => {
                    break;
                }
            }
        }
        if let 'e' | 'E' = self.peek() {
            self.next(); //consume 'E'
            e = self.parse_integer();
        }

        Number::new(d, e)
    }

    pub fn parse_number(&mut self, c: char) -> Number {
        self.parse_decimal(c)
    }
}


pub fn is_id_start(c: char) -> bool {
    // This is XID_Start OR '_' (which formally is not a XID_Start).
    c == '_' || unicode_xid::UnicodeXID::is_xid_start(c)
}

pub fn is_id_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}


#[cfg(test)]
mod tests {
    use crate::tokenizer::cursor::Cursor;

    #[test]
    fn test_integer() {
        let text = "12345";
        let mut cur = Cursor::new(text);
        let i = cur.parse_integer();
        assert_eq!(i, 12345);

        let text = "-12345";
        let mut cur = Cursor::new(text);
        let i = cur.parse_integer();
        assert_eq!(i, -12345);

        let text = "12_345";
        let mut cur = Cursor::new(text);
        let i = cur.parse_integer();
        assert_eq!(i, 12345);
    }
    #[test]
    fn test_decimal() {
        test_number("123.45", 123.45, 0);
        test_number("12_3.45", 123.45, 0);
        test_number("12_3.4_5", 123.45, 0);
        test_number("123E5", 123.0, 5);
        test_number("123E+5", 123.0, 5);
        test_number("123e-5", 123.0, -5);
        test_number("123e0_5", 123.0, 5);
    }

    fn test_number(text: &str, sig: f64, exp: i32) {
        let mut cur = Cursor::new(text);
        let c = cur.next().unwrap();
        let n = cur.parse_decimal(c);
        assert_eq!(n.significand, sig);
        assert_eq!(n.exponent, exp);
    }

    #[test]
    fn test_newline() {
        let text = "first word\nsecond word and\n  third line \n  \n  fifth?\n";
        let mut cur = Cursor::new(text);
        cur.next_token();
        assert!(cur.newline_found);
        cur.next_token();
        assert!(!cur.newline_found);

        cur.next_token();
        assert!(cur.newline_found);
        cur.next_token();
        assert!(!cur.newline_found);
        cur.next_token();
        assert!(!cur.newline_found);

        cur.next_token();
        assert!(cur.newline_found);
        cur.next_token();
        assert!(!cur.newline_found);

        cur.next_token();
        assert!(cur.newline_found);
    }
}
