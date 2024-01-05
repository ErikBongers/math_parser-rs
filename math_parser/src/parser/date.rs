use crate::errors::Error;
use crate::parser::date::date::{EMPTY_YEAR, Month};
use crate::tokenizer::cursor::Range;

pub mod date {
    use serde::Serialize;

    #[derive(Clone, Copy, Debug, Serialize, PartialEq)]
    pub enum Month {JAN = 1, FEB = 2, MAR = 3, APR = 4, MAY = 5, JUN = 6, JUL = 7, AUG = 8, SEP = 9, OCT = 10, NOV = 11, DEC = 12, NONE = 0}
    #[derive(Clone, Copy)]
    pub enum DateFormat { YMD, DMY, MDY, Undefined}
    pub const LAST: i8 = 99;
    pub const EMPTY_YEAR: i32 = i32::MIN; //TODO: magic value.

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
                DateFormat::Undefined => (2,1,0) //default.
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


}

#[derive(Clone)]
pub struct Date { //TODO: rename to Timepoint
    pub month: Month,
    pub day: i8,
    pub year: i32,
    pub range: Range,
    pub errors: Vec<Error>,
}

impl Date {
    pub fn new() -> Self {
        Date {
            month: Month::NONE,
            day: 0,
            year: EMPTY_YEAR,
            range: Range::none(),
            errors: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.year == EMPTY_YEAR { return false; }
        if self.day <= 0 || self.day > 31 { return false; }
        if self.month == Month::NONE { return false; }

        true
    }
}