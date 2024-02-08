use std::cmp::max;
use std::ops::{Add, Div, Mul, Sub};
use crate::errors;
use crate::errors::Error;
use crate::globals::Globals;
use crate::resolver::scope::DecimalChar;
use crate::resolver::unit::{Unit, UnitsView};
use crate::resolver::value::NumberFormat;
use crate::tokenizer::cursor::Range;

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
            unit : Unit::none(),
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

    /// converts a Number to an f64 where NaN is replaced with 0.0
    pub fn sortable_value(&self) -> f64 {
        if self.to_double().is_nan() { 0.0 }
        else { self.to_double() }
    }

    pub fn normalize_number (&self) -> Number {
        //don't use to_double() to avoid loss of precision.

        //find base of sig, to get one digit before the decimal.
        let mut sig_base: f64 = 0.0;
        let mut sig = self.significand;
        loop {
            if 1.0 <= sig && sig < 10.0 { break }
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

    pub fn convert_to_exponent(&mut self, e: i32) {
        while self.exponent < e {
            self.exponent += 1;
            self.significand /= 10.0;
        }
        while self.exponent > e {
            self.exponent -= 1;
            self.significand *= 10.0;
        }
    }

    pub fn to_si(&self, globals: &Globals) -> Number {
        let mut num = self.clone();
        if globals.unit_defs.contains_key(&*self.unit.id) {
            num.significand = globals.unit_defs[&*self.unit.id].convert_to_si(self.to_double());
            num.exponent = 0;
            num.unit = Unit::from_id(globals.unit_defs[&*self.unit.id].si_id, None);
        } else {
            //ignore
        }
        num.convert_to_exponent(self.exponent);
        num
    }

    pub fn convert_to_unit(&mut self, to: &Unit, units_view: &UnitsView, range: &Range, errors: &mut Vec<Error>, globals: &Globals) {
        if self.unit.is_empty() {
            self.unit = to.clone();
            if let None = units_view.get_def(&to.id, globals) {
                errors.push(errors::unit_not_def(&to.id, to.range.as_ref().unwrap_or(range).clone()));
            }
            return;
        }
        if let None = units_view.get_def(&self.unit.id, globals) {
            return; //should already have been detected.
        }
        if let None = units_view.get_def(&to.id, globals) {
            errors.push(errors::unit_not_def(&to.id, to.range.as_ref().unwrap_or(range).clone()));
            return;
        }
        if units_view.get_def(&self.unit.id, globals).unwrap().property != units_view.get_def(&to.id, globals).unwrap().property { //unit ids already checked.
            errors.push(errors::unit_prop_diff(range.clone()));
            return;
        }
        let si_val = units_view.get_def(&self.unit.id, globals).unwrap().convert_to_si(self.to_double()); //unit ids already checked.
        let val = units_view.get_def(&to.id, globals).unwrap().convert_from_si(si_val); //unit ids already checked.
        self.significand = val;
        self.unit = to.clone();
        let exponent = self.exponent;
        self.exponent = 0;
        self.convert_to_exponent(exponent);
    }

    #[inline]
    pub fn to_double(&self) -> f64 {
        let base: f64 = 10.0;
        self.significand * base.powf(self.exponent as f64)
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        let d = self.to_double();
        d == d.trunc()
    }
}

impl Add for &Number {
    type Output = Number;

    fn add(self, rhs: Self) -> Self::Output {
        let max_exponent = max(self.exponent, rhs.exponent);
        let mut n1 = self.clone();
        n1.convert_to_exponent(max_exponent);
        let mut n2 = rhs.clone();
        n2.convert_to_exponent(max_exponent);
        n1.significand += n2.significand;
        n1
    }
}

impl Sub for &Number {
    type Output = Number;

    fn sub(self, rhs: Self) -> Self::Output {
        let max_exponent = max(self.exponent, rhs.exponent);
        let mut n1 = self.clone();
        n1.convert_to_exponent(max_exponent);
        let mut n2 = rhs.clone();
        n2.convert_to_exponent(max_exponent);
        n1.significand -= n2.significand;
        n1
    }
}

impl Mul for &Number {
    type Output = Number;

    fn mul(self, rhs: Self) -> Self::Output {
        Number {
            significand: self.significand*rhs.significand,
            exponent: self.exponent +rhs.exponent,
            unit: if rhs.unit.is_empty() {
                self.unit.clone()
            } else {
                if self.unit.is_empty() {
                    rhs.unit.clone()
                } else {
                    Unit::none() //reserved for combined units. Don't report an error.
                }
            },
            fmt: self.fmt.clone(),
        }
    }
}

impl Div for &Number {
    type Output = Number;

    fn div(self, rhs: Self) -> Self::Output {
        Number {
            significand: self.significand/rhs.significand,
            exponent: self.exponent-rhs.exponent,
            unit: if rhs.unit.is_empty() {
                self.unit.clone()
            } else {
                Unit::none() //reserved for combined units. Don't report an error.
            },
            fmt: self.fmt.clone(),
        }
    }
}

fn find_decimal_char(stream: &str, range: &Range) -> Result<DecimalChar, Error> {
    let dots: Vec<_> = stream.match_indices('.').map(|tupple| tupple.0).collect();
    let commas: Vec<_> = stream.match_indices(',').map(|tupple| tupple.0).collect();
    if dots.len() + commas.len() == 0 {
        return Ok(DecimalChar::Dot); //only digits, so irrelevant separator
    }
    if dots.len() + commas.len() < 2 {
        return Err(errors::inv_number_str("ambiguous decimal point or thousands separator", range.clone()));
    }
    if dots.last().unwrap() > commas.first().unwrap() && commas.last().unwrap() > dots.first().unwrap() {
        return Err(errors::inv_number_str("mixed dots and commas", range.clone()));
    }
    if dots.first().unwrap() > commas.first().unwrap() {
        Ok(DecimalChar::Dot)
    } else {
        Ok(DecimalChar::Comma)
    }
}

pub fn parse_formatted_number(stream: &str, range: &Range, decimal_char: DecimalChar) -> Result<Number, Error> {
    let (decimal_char, thou_char) = match decimal_char {
        DecimalChar::Comma => { (',', '.') },
        DecimalChar::Dot => { ('.', ',') },
        DecimalChar::Auto => {
            return parse_formatted_number(stream, range, find_decimal_char(stream, range)?);
        }
    };
    let mut decimal_divider = 1.0;
    let chars = stream.chars();
    let mut d: f64 = 0.0;
    for c in chars {
        if c >= '0' && c <= '9' {
            if decimal_divider == 1.0 {
                d = d * 10.0 + (c as i32 - '0' as i32) as f64;
            } else {
                d += (c as i32 - '0' as i32) as f64/ decimal_divider;
                decimal_divider *= 10.0;
            }
        } else {
            if c == thou_char {
                if decimal_divider != 1.0 {
                    return Err(errors::inv_number_str("thousands divider char not allowed after decimal point", range.clone()));
                }
                //note that the thou_char is currently allowed everywhere before the decimal_char !
            } else {
                if c == decimal_char {
                    if decimal_divider == 1.0 {
                        decimal_divider = 10.0;
                    } else {
                        return Err(errors::inv_number_str("decimal point encountered more than once", range.clone()));
                    }
                } else {
                    return Err(errors::inv_number_str("unexpected character", range.clone()));
                }
            }
        }
    }
    Ok(Number {
        significand: d,
        exponent: 0,
        unit: Unit::none(),
        fmt: NumberFormat::Dec,
    })
}