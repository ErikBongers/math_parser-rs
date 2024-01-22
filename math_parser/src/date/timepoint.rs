use std::ops;
use crate::errors::Error;
use crate::tokenizer::cursor::Range;

use std::fmt;
use serde::Serialize;
use crate::date::Duration;

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum Month {JAN = 1, FEB = 2, MAR = 3, APR = 4, MAY = 5, JUN = 6, JUL = 7, AUG = 8, SEP = 9, OCT = 10, NOV = 11, DEC = 12, NONE = 0}
#[derive(Clone, Copy, Debug)]
pub enum DateFormat { YMD, DMY, MDY}
impl fmt::Display for DateFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub struct DateFormatIndices {
    pub day: usize,
    pub month: usize,
    pub year: usize,
}

impl DateFormat {
    pub fn indices(&self) -> DateFormatIndices {
        let (day, month, year) = match self {
            DateFormat::DMY => (0,1,2),
            DateFormat::MDY => (1,0,2),
            DateFormat::YMD => (2,1,0),
        };
        DateFormatIndices { day, month, year }
    }
}

///assumes lower case.
pub fn month_from_str(text: &str) -> Month {
    match text {
        "jan" => Month::JAN,
        "feb" => Month::FEB,
        "mar" => Month::MAR,
        "apr" => Month::APR,
        "may" => Month::MAY,
        "jun" => Month::JUN,
        "jul" => Month::JUL,
        "aug" => Month::AUG,
        "sep" => Month::SEP,
        "oct" => Month::OCT,
        "nov" => Month::NOV,
        "dec" => Month::DEC,

        _ => Month::NONE
    }
}

pub fn month_from_int(i: i32) -> Month {
    match i {
        1 => Month::JAN,
        2 => Month::FEB,
        3 => Month::MAR,
        4 => Month::APR,
        5 => Month::MAY,
        6 => Month::JUN,
        7 => Month::JUL,
        8 => Month::AUG,
        9 => Month::SEP,
        10 => Month::OCT,
        11 => Month::NOV,
        12 => Month::DEC,

        _ => Month::NONE
    }
}

#[derive(Clone)]
pub enum Day {
    Value(i8),
    Last,
    None
}

impl Day {
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(*self, Day::None)
    }
    #[inline]
    pub const fn is_last(&self) -> bool {
        matches!(*self, Day::Last)
    }
    #[inline]
    pub const fn unwrap_or(&self, default: i8) -> i8 {
        return if let Day::Value(day) = *self {
            day
        } else {
            default
        }
    }
}

#[derive(Clone)]
pub struct Timepoint {
    pub month: Month,
    pub day: Day,
    pub year: Option<i32>,
    pub range: Range,
    pub errors: Vec<Error>,
}

impl Timepoint {
    pub fn new() -> Self {
        Timepoint {
            month: Month::NONE,
            day: Day::None,
            year: None,
            range: Range::none(),
            errors: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.year.is_none() { return false; }
        if self.day.is_none() { return false; }
        if let Day::Value(day) = self.day {
            if day <= 0 || day > 31 { return false; }
        }
        if self.month == Month::NONE { return false; }

        true
    }

    ///Always returns a number, but 0 if undetermined.
    pub fn get_normalized_day(&self) -> i8 {
        //check leap year: if y/4 and not y/100 : leap year. Also, if y/100 and y/400: leap year.
        if  self.day.is_last() {
            match self.month {
                Month::APR | Month::JUN | Month::SEP | Month::NOV => 30,
                Month::FEB => {
                    if let Some(year) = self.year {
                        if (year%4 == 0 && year%100 != 0)
                            || (year%100 == 0 && year%400 == 0) {
                            29
                        } else {
                            28
                        }
                    } else {
                        self.day.unwrap_or(0)
                    }
                },
                Month::NONE => self.day.unwrap_or(0),
                _ => 31
            }
        } else {
            self.day.unwrap_or(0)
        }
    }
}

impl ops::Sub<&Timepoint> for &Timepoint {
    type Output = Duration;

    fn sub(self, rhs: &Timepoint) -> Self::Output {
        Duration {
            days: self.get_normalized_day() as i32 - rhs.get_normalized_day() as i32,
            months: self.month as i32 - rhs.month as i32,
            years: self.year.unwrap_or(0) - rhs.year.unwrap_or(0)
        }
    }
}