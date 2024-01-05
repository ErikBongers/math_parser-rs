use crate::errors::{Error, ERROR_MAP, ErrorId, ErrorType};
use crate::parser::date::{Date, date};
use crate::parser::date::date::{DateFormat, EMPTY_YEAR, LAST, Month, month_from_int, month_from_str};
use crate::tokenizer::cursor::Range;


pub fn parse_date_string(text: &str, range: &Range) -> Date {
    let slices = text.split(|c| c == ' ' || c == '/' || c == ',' || c == '-').filter(|s| !s.is_empty()).collect();
    let mut parser = FormattedDateParser::new(&slices, range);
    parser.parse()
}

struct DateParserState<'s, 'r> {
    date: Date,
    range: &'r Range,
    ymd_format: bool,
    date_format: DateFormat,
    slices: &'s Vec<&'s str>,
}

pub struct FormattedDateParser<'s, 'r> {
    slices: &'s Vec<&'s str>,
    parser_state: DateParserState<'s, 'r>,
}

impl<'s, 'r> FormattedDateParser<'s, 'r> {
   pub fn new(slices: &'s Vec<&'s str>, range: &'r Range) -> Self {
        FormattedDateParser {
            slices,
            parser_state:
            DateParserState {
                date: Date::new(),
                range,
                ymd_format: false,
                slices: &slices,
                date_format: DateFormat::DMY,
            },
        }
    }
    pub fn parse(&mut self) -> Date {
        for (i, slice) in self.slices.iter().enumerate() {
            self.parser_state.parse_any_slice(i, slice);
            if has_real_errors(&self.parser_state.date.errors) {
                self.parser_state.parse_for_format();
                return self.parser_state.date.clone();
            }
        }
        if !self.parser_state.date.is_valid() {
            self.parser_state.parse_for_format()
        }
        self.parser_state.date.clone()
    }
}
impl<'s, 'r> DateParserState<'s, 'r> {
    fn parse_any_slice(&mut self, slice_no: usize, slice: &str) {
        if slice == "last" {
            if self.date.day != 0 {
                self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["multiple values for day."]));
            } else {
                self.date.day = date::LAST;
            }
            return;
        }
        let low_slice = slice.to_lowercase();
        let month = date::month_from_str(low_slice.as_str());
        if month != Month::NONE {
            if self.date.month != Month::NONE {
                self.date.month = month;
                return;
            } else {
                self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["multiple values for month."]));

            }
        }
        //from here on, it should be all numbers.
        let Ok(n) = slice.parse::<i32>() else {
            self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["invalid numeric value."]));
            return;
        };
        if n < 0 {
            self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["invalid numeric value."]));
            return;
        }
        if n > 31 {
            if self.date.year != EMPTY_YEAR {
                self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["multiple values for year."]));
                return ;
            } else {
                self.date.year = n;
                if slice_no == 0 {
                    self.ymd_format = true;
                }
            }
        } else {
            if n > 12 {
                if self.count_date_slices() >= 3 && !self.has_year_slice() {
                    self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["values could be month or year."]));
                    return;
                }
                if self.date.day != 0 {
                    self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["multiple values for day."]));
                    return;
                }
                self.date.day = n as i8;
            } else { // n <= 12
                if self.count_date_slices() == 3 {
                    if self.count_same_date_values(n) == 3 {
                        self.date.day = n as i8;
                        self.date.month = month_from_int(n);
                        self.date.year = n;
                        return;
                    }
                    //slice could be day, month, year
                    if self.has_year_slice() {
                        if self.has_month_slice() {
                            if self.date.day == 0 {
                                self.date.day = n as i8;
                            } else {
                                self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["not clear which value is day or month."]));
                            }
                            return;
                        }
                        if self.has_day_slice() {
                            if self.date.month == Month::NONE {
                                self.date.month = month_from_int(n);
                            } else {
                                self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["not clear which value is day or month."]));
                            }
                            return;
                        }
                        if self.count_same_date_values(n) >= 2 {
                            self.date.day = n as i8;
                            self.date.month = month_from_int(n);
                            return;
                        }
                        self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["not clear which value is day or month."]));
                    }
                    if self.ymd_format {
                        self.parse_ymd_slice_number(slice_no, n);
                    }
                } else {
                    if self.count_date_slices() == 2 {
                        if self.has_day_slice() {
                            self.date.month = month_from_int(n);
                            return;
                        }
                        if self.has_month_slice() {
                            self.date.day = n as  i8;
                            return;
                        }
                        self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["not clear which value is day or month."]));
                    }
                }
            }
        }
    }

    fn parse_ymd_slice_number(&mut self, slice_no: usize, n: i32) {
        match slice_no {
            0 => {
                if self.date.year == EMPTY_YEAR {
                self.date.year = n
                } else {
                self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), & ["assuming ymd format, but day is already filled."]));
                }
            },
            1 => {
                if self.date.month == Month::NONE {
                    self.date.month = month_from_int(n);
                } else {
                    self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), & ["assuming ymd but month is already filled."]));
                }
            },
            2 => {
                if self.date.day == 0 {
                    self.date.day = n as i8;
                } else {
                    self.date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), & ["assuming ymd but day is already filled."]));
                }
            },
            _ => () //ignore
        }
    }

    fn has_year_slice(&self) -> bool {
        self.slices.iter().find(|s| s.parse::<i32>().unwrap_or(0) > 31).is_some() //TODO: first exclude the time slices!
    }
    fn has_day_slice(&self) -> bool {
        self.slices.iter()//TODO: first exclude the time slices!
            .find(|s| {
                if **s == "last" { return true; }
                let n = s.parse::<i32>().unwrap_or(0);
                n > 12 && n <= 31
            })
            .is_some()
    }

    fn has_month_slice(&self) -> bool {
        self.slices.iter()
            .find(|s| {
                let lower = s.to_lowercase();
                date::month_from_str(&lower) != Month::NONE
            })
            .is_some() //TODO: first exclude the time slices!
    }


    fn count_date_slices(&self) -> usize {
        self.slices.len() //TODO: exlcude time slices.
    }

    fn count_same_date_values(&self, n: i32) -> usize {
        //TODO: exlcude time slices.
        self.slices.iter().filter(|s| s.parse::<i32>().unwrap_or(-n) == n).count()
    }

    fn parse_for_format(&mut self) {
        if self.slices.len() != 3 {
            return;
        }
        let idx = self.date_format.indices();
        let mut tmp_date = Date::new();
        self.parse_year(&mut tmp_date, self.slices[idx.year]);
        self.parse_month(&mut tmp_date, self.slices[idx.month]);
        self.parse_day(&mut tmp_date, self.slices[idx.day]);

        if !tmp_date.is_valid() {
            tmp_date.errors.push(Error::build(ErrorId::InvDateStr, self.range.clone(), &["invalid date for format."])); //TODO specify the format.
        }
        self.date = tmp_date;
   }

    fn parse_day(&self, date: &mut Date, slice: &str) {
        if slice == "last" {
            date.day = LAST;
            return;
        }
        date.day = slice.parse::<i8>().unwrap_or(0);
    }

    fn parse_year(&self, date: &mut Date, slice: &str) {
        date.year = slice.parse::<i32>().unwrap_or(EMPTY_YEAR);
    }

    fn parse_month(&self, date: &mut Date, slice: &str) {
        let month = month_from_str(&slice.to_lowercase());
        if month != Month::NONE {
            date.month = month;
            return;
        }
        let Ok(n) = slice.parse::<i32>() else {
            date.month = Month::NONE;
            return;
        };
        date.month = month_from_int(n);
   }
}

//TODO: move this elsewhere
fn has_real_errors(errors: &Vec<Error>) -> bool {
    errors.iter().find(|e| ERROR_MAP.get(&e.id).unwrap().error_type == ErrorType::E).is_some()
}

