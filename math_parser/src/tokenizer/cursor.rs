use std::cmp::{max, min};
use std::ops;
use std::str::Chars;
use serde::Serialize;
use crate::errors::{Error, ErrorId};
use crate::resolver::add_error;
use crate::resolver::globals::Globals;
use crate::resolver::unit::{Unit, UnitsView};
use crate::resolver::value::{NumberFormat, Value};

#[derive(Clone)]
pub struct Cursor<'a> {
    pub text: &'a str,
    pub chars: Chars<'a>,
    pub len_text: usize,
    pub newline_found: bool,
    pub number: Number,
    pub is_beginning_of_text: bool,
}

#[derive(Debug, Clone, Serialize)] //TODO: make Copy instead of Clone?
pub struct Range {
    pub source_index: u8,
    pub start :usize,
    pub end : usize
}

impl Range {
    pub fn none() -> Range {
        Range {
            source_index: 0,
            start: 0,
            end: 0
        }
    }
}

impl ops::AddAssign for Range {
    fn add_assign(&mut self, rhs: Self) {
        self.start = min(self.start, rhs.start);
        self.end = max(self.end, rhs.end);
        assert_eq!(self.source_index, rhs.source_index);
    }
}

impl ops::Add for &Range {
    type Output = Range;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.source_index, rhs.source_index);
        Range {
            source_index: self.source_index,
            start: min(self.start, rhs.start),
            end: max(self.end, rhs.end),
        }
    }
}


#[derive(Clone)]
pub struct Number {
    pub significand: f64,
    pub exponent: i32,
    pub unit: Unit,
    pub fmt: NumberFormat
}

impl Number {
    pub fn new(significand: f64, exponent: i32) -> Self {
        Number {
            significand,
            exponent,
            unit : Unit { range: None, id: "".to_string() },
            fmt: NumberFormat::Dec
        }
    }

    pub fn from(n: f64) -> Number {
        Number {
            significand: n,
            exponent: 0,
            unit: Unit::none(),
            fmt: NumberFormat::Dec
        }
    }

    pub fn normalize_number (&self) -> Number {
        //don't use to_double() to avoid loss of precision.

        //find base of sig, to get one digit before the decimal.
        let mut sig_base: f64 = 0.0;
        let mut sig = self.significand;
        loop {
            if(1.0 <= sig && sig < 10.0) { break }
            if sig >= 10.0 {
                sig_base += 1.0;
                sig /= 10.0;
            } else { // < 1.0
                sig_base -= 1.0;
                sig *= 10.0;
            }
        }
        //now we have sig's base, but there's already an exponent.

        Number::new(self.significand/10_f64.powf(sig_base), self.exponent+(sig_base as i32))
    }



    pub fn to_si(&self, globals: &Globals) -> f64 {
        if globals.unit_defs.contains_key(&*self.unit.id) {
            let to_si = globals.unit_defs[&*self.unit.id].to_si;
            to_si(&globals.unit_defs[&*self.unit.id], self.to_double())
        } else {
            self.to_double()
        }
    }

    pub fn convert_to_unit(&mut self, to: &Unit, units_view: &UnitsView, range: &Range, errors: &mut Vec<Error>, globals: &Globals) {
        if self.unit.is_empty() {
            self.unit = to.clone();
            if let None = units_view.get_def(&to.id, globals) {
                add_error(errors, ErrorId::UnitNotDef, range.clone(), &[&to.id], Value::error(range));
            }
            return;
        }
        if let None = units_view.get_def(&self.unit.id, globals) {
            return; //should already have been detected.
        }
        if let None = units_view.get_def(&to.id, globals) {
            add_error(errors, ErrorId::UnitNotDef, range.clone(), &[&to.id], Value::error(range));
            return;
        }
        if units_view.get_def(&self.unit.id, globals).unwrap().property != units_view.get_def(&to.id, globals).unwrap().property {
            add_error(errors, ErrorId::UnitPropDiff, range.clone(), &[""], Value::error(range));
            return;
        }
        let to_si = units_view.get_def(&self.unit.id, globals).unwrap().to_si;
        let from_si = units_view.get_def(&to.id, globals).unwrap().from_si;
        let si_val = to_si(&units_view.get_def(&self.unit.id, globals).unwrap(), self.significand); //TODO: call this function directly on UnitDef?
        let val = from_si(&units_view.get_def(&to.id, globals).unwrap(), si_val);
        self.significand = val;
        self.unit = to.clone();
    }

    pub fn to_double(&self) -> f64 {
        let base: f64 = 10.0;
        self.significand * base.powf(self.exponent as f64)
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

    pub fn copy_from(&mut self, cursor: &Cursor<'a>) {
        self.text = cursor.text;
        self.chars = cursor.chars.clone();
        self.len_text = cursor.len_text;
        self.newline_found = cursor.newline_found;
        self.number = cursor.number.clone();
        self.is_beginning_of_text = cursor.is_beginning_of_text;
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
        if c == '0' {
            match self.peek() {
               'b' | 'B' => self.parse_binary(),
                'x' | 'X' => self.parse_hex(),
                'o' | 'O' => self.parse_oct(),
                _ => self.parse_decimal(c)
            }
       } else {
            self.parse_decimal(c)
        }
    }

    pub fn parse_binary(&mut self) -> Number {
        let mut bin: u64 = 0;
        self.next(); //consume 'b'\
        loop {
            if self.peek() != '0' && self.peek() != '1' && self.peek() != '_' {
                break;
            }
            let c = self.next().unwrap(); //checked.
            if c == '_' { continue;}
            bin <<= 1;
            if c == '1' {
                bin += 1;
            }
        }
        Number { significand: bin as f64, exponent: 0, unit: Unit::none(), fmt: NumberFormat::Bin, }
    }

    pub fn parse_oct(&mut self) -> Number {
        let mut oct: u64 = 0;
        self.next(); //consume 'b'
        loop {
            if (self.peek() < '0' || self.peek() > '7') && self.peek() != '_' {
                break;
            }
            let c = self.next().unwrap(); //checked
            if c == '_' { continue;}
            oct *= 8;
            oct += c as u64 - '0' as u64;
        }
        Number { significand: oct as f64, exponent: 0, unit: Unit::none(), fmt: NumberFormat::Oct, }
    }

    pub fn parse_hex(&mut self) -> Number {
        let mut hex: u64 = 0;
        self.next(); //consume 'b'\
        loop {
            let c = match self.peek() {
                '_' => continue,
                c if c >= '0' && c <= '9' => c as u64 - '0' as u64,
                c if c >= 'a' && c <= 'f' => c as u64 - 'a' as u64 + 10,
                c if c >= 'A' && c <= 'F' => c as u64 - 'A' as u64 + 10,
                _ => break
            };
            self.next();
            hex *= 16;
            hex += c;
        }
        Number { significand: hex as f64, exponent: 0, unit: Unit::none(), fmt: NumberFormat::Hex, }
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
