use crate::errors::{Error, ErrorId};
use crate::globals::Globals;
use crate::resolver::add_error;
use crate::resolver::scope::Scope;
use crate::resolver::unit::{Unit, UnitsView};
use crate::resolver::value::{NumberFormat, Value};
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
            unit : Unit { id: "".to_string() },
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



    pub fn to_si(&self, globals: &Globals) -> f64 {
        if globals.unit_defs.contains_key(&*self.unit.id) {
            globals.unit_defs[&*self.unit.id].convert_to_si(self.to_double())
        } else {
            self.to_double()
        }
    }

    pub fn convert_to_unit(&mut self, to: &Unit, units_view: &UnitsView, range: &Range, errors: &mut Vec<Error>, globals: &Globals) {
        if self.unit.is_empty() {
            self.unit = to.clone();
            if let None = units_view.get_def(&to.id, globals) {
                add_error(errors, ErrorId::UnitNotDef, range.clone(), &[&to.id], Value::error(range.clone()));
            }
            return;
        }
        if let None = units_view.get_def(&self.unit.id, globals) {
            return; //should already have been detected.
        }
        if let None = units_view.get_def(&to.id, globals) {
            add_error(errors, ErrorId::UnitNotDef, range.clone(), &[&to.id], Value::error(range.clone()));
            return;
        }
        if units_view.get_def(&self.unit.id, globals).unwrap().property != units_view.get_def(&to.id, globals).unwrap().property {
            add_error(errors, ErrorId::UnitPropDiff, range.clone(), &[""], Value::error(range.clone()));
            return;
        }
        let si_val = units_view.get_def(&self.unit.id, globals).unwrap().convert_to_si(self.to_double());
        let val = units_view.get_def(&to.id, globals).unwrap().convert_from_si(si_val);
        self.significand = val;
        self.exponent = 0;
        self.unit = to.clone();
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

pub fn parse_formatted_number<'s>(stream: &str, range: &Range, scope: &Scope) -> Result<Number, Error> {
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
            if c == scope.thou_char {
                if decimal_divider != 1.0 {
                    return Err(Error{
                        id: ErrorId::InvNumberStr,
                        message: "thousands divider char not allowed after decimal point".to_string(),
                        range: range.clone(),
                        stack_trace: None,
                    });
                }
                //note that the thou_char is currently allowed everywhere before the decimal_char !
            } else {
                if c == scope.decimal_char {
                    if decimal_divider == 1.0 {
                        decimal_divider = 10.0;
                    } else {
                        return Err(Error{
                            id: ErrorId::InvNumberStr,
                            message: "decimal point encountered more than once".to_string(),
                            range: range.clone(),
                            stack_trace: None,
                        });
                    }
                } else {
                    return Err(Error{
                        id: ErrorId::InvNumberStr,
                        message: "unexpected character".to_string(),
                        range: range.clone(),
                        stack_trace: None,
                    });
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