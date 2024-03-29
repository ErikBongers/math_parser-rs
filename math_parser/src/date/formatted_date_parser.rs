use errors::has_real_errors;
use crate::errors;
use crate::date::{Timepoint, Day};
use crate::date::{DateFormat, Month, month_from_int, month_from_str};
use crate::tokenizer::cursor::Range;


pub fn parse_date_string(text: &str, range: &Range, date_format: DateFormat) -> Timepoint {
    let slices = text.split(|c| c == ' ' || c == '/' || c == ',' || c == '-').filter(|s| !s.is_empty()).collect();
    let mut parser = FormattedDateParser::new(&slices, range);
    parser.parser_state.date_format = date_format;
    parser.parse()
}

struct DateParserState<'s, 'r> {
    date: Timepoint,
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
                date: Timepoint::new(),
                range,
                ymd_format: false,
                slices: &slices,
                date_format: DateFormat::DMY,
            },
        }
    }
    pub fn parse(&mut self) -> Timepoint {
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
            if !self.date.day.is_none() {
                self.date.errors.push(errors::inv_date_str("multiple values for day.", self.range.clone()));
            } else {
                self.date.day = Day::Last;
            }
            return;
        }
        let low_slice = slice.to_lowercase();
        let month = month_from_str(low_slice.as_str());
        if month != Month::NONE {
            if self.date.month == Month::NONE {
                self.date.month = month;
                return;
            } else {
                self.date.errors.push(errors::inv_date_str("multiple values for month.", self.range.clone()));

            }
        }
        //from here on, it should be all numbers.
        let Ok(n) = slice.parse::<i32>() else {
            self.date.errors.push(errors::inv_date_str("invalid numeric value.", self.range.clone()));
            return;
        };
        if n < 0 {
            self.date.errors.push(errors::inv_date_str("invalid numeric value.", self.range.clone()));
            return;
        }
        if n > 31 {
            if self.date.year.is_some() {
                self.date.errors.push(errors::inv_date_str("multiple values for year.", self.range.clone()));
                return ;
            } else {
                self.date.year = Some(n);
                if slice_no == 0 {
                    self.ymd_format = true;
                }
            }
        } else {
            if n > 12 {
                if self.count_date_slices() >= 3 && !self.has_year_slice() {
                    self.date.errors.push(errors::inv_date_str("values could be month or year.", self.range.clone()));
                    return;
                }
                if !self.date.day.is_none() {
                    self.date.errors.push(errors::inv_date_str("multiple values for day.", self.range.clone()));
                    return;
                }
                self.date.day = Day::Value(n as i8);
            } else { // n <= 12
                if self.count_date_slices() == 3 {
                    if self.count_same_date_values(n) == 3 {
                        self.date.day = Day::Value(n as i8);
                        self.date.month = month_from_int(n);
                        self.date.year = Some(n);
                        return;
                    }
                    //slice could be day, month, year
                    if self.has_year_slice() {
                        if self.has_month_slice() {
                            if self.date.day.is_none() {
                                self.date.day = Day::Value(n as i8);
                            } else {
                                self.date.errors.push(errors::inv_date_str("not clear which value is day or month.", self.range.clone()));
                            }
                            return;
                        }
                        if self.has_day_slice() {
                            if self.date.month == Month::NONE {
                                self.date.month = month_from_int(n);
                            } else {
                                self.date.errors.push(errors::inv_date_str("not clear which value is day or month.", self.range.clone()));
                            }
                            return;
                        }
                        if self.count_same_date_values(n) >= 2 {
                            self.date.day = Day::Value(n as i8);
                            self.date.month = month_from_int(n);
                            return;
                        }
                        self.date.errors.push(errors::inv_date_str("not clear which value is day or month.", self.range.clone()));
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
                            self.date.day = Day::Value(n as  i8);
                            return;
                        }
                        self.date.errors.push(errors::inv_date_str("not clear which value is day or month.", self.range.clone()));
                    }
                }
            }
        }
    }

    fn parse_ymd_slice_number(&mut self, slice_no: usize, n: i32) {
        match slice_no {
            0 => {
                if self.date.year.is_none() {
                self.date.year = Some(n)
                } else {
                self.date.errors.push(errors::inv_date_str("assuming ymd format, but day is already filled.", self.range.clone()));
                }
            },
            1 => {
                if self.date.month == Month::NONE {
                    self.date.month = month_from_int(n);
                } else {
                    self.date.errors.push(errors::inv_date_str("assuming ymd but month is already filled.", self.range.clone()));
                }
            },
            2 => {
                if self.date.day.is_none() {
                    self.date.day = Day::Value(n as i8);
                } else {
                    self.date.errors.push(errors::inv_date_str("assuming ymd but day is already filled.", self.range.clone()));
                }
            },
            _ => () //ignore
        }
    }

    fn has_year_slice(&self) -> bool {
        self.slices.iter().find(|s| s.parse::<i32>().unwrap_or(0) > 31).is_some()
    }
    fn has_day_slice(&self) -> bool {
        self.slices.iter()
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
                month_from_str(&lower) != Month::NONE
            })
            .is_some()
    }


    fn count_date_slices(&self) -> usize {
        self.slices.len()
    }

    fn count_same_date_values(&self, n: i32) -> usize {
        self.slices.iter().filter(|s| s.parse::<i32>().unwrap_or(-n) == n).count()
    }

    fn parse_for_format(&mut self) {
        if self.slices.len() != 3 {
            return;
        }
        let idx = self.date_format.indices();
        let mut tmp_date = Timepoint::new();
        self.parse_year(&mut tmp_date, self.slices[idx.year]);
        self.parse_month(&mut tmp_date, self.slices[idx.month]);
        self.parse_day(&mut tmp_date, self.slices[idx.day]);

        if !tmp_date.is_valid() {
            tmp_date.errors.push(errors::inv_date_str_for_format(self.date_format.to_string().as_str(), self.range.clone()));
        }
        self.date = tmp_date;
   }

    fn parse_day(&self, date: &mut Timepoint, slice: &str) {
        if slice == "last" {
            date.day = Day::Last;
            return;
        }
        date.day = slice
            .parse::<i8>()
            .map_or(Day::None,
                    |d| Day::Value(d));
    }

    fn parse_year(&self, date: &mut Timepoint, slice: &str) {
        date.year = slice
            .parse::<i32>()
            .map_or(None,
                    |y| Some(y));
    }

    fn parse_month(&self, date: &mut Timepoint, slice: &str) {
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

