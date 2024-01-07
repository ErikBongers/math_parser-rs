use crate::errors::{Error, ErrorId};
use crate::tokenizer::cursor::{Number, Range};

pub mod date {
    use crate::errors::{Error, ErrorId};
    use crate::tokenizer::cursor::{Number, Range};

    #[derive(Clone, Copy)]
    pub struct Duration {
        pub days: i32,
        pub months: i32,
        pub years: i32,
    }

    impl Duration {
        pub fn new() -> Duration {
            Duration {
                days: 0,
                months: 0,
                years: 0,
            }
        }

        //assumes 1 month = 30 days
        pub fn normalize(&mut self) {
            if self.days >= 0 && self.days <= 31 &&
                self.months >= 0 && self.months <= 12 &&
                self.years >= 0 {
                return;
            }
            self.days = self.to_days(); //todo: don't use to_days(), to avoid accumulation of error: 1. if days is negative, subract from months, if months is negative, subtract from years. If years is negative...do the to_days() thing...
            self.months = 0;
            self.years = 0;

            self.years = (self.days as f64 / 365.2425).floor() as i32;
            self.days = (self.days as f64 % 365.2425) as i32;
            self.months = (self.days as f64 / 30.437) as i32;
            self.days = (self.days as f64 % 30.437) as i32;
        }

        fn to_days(&self) -> i32 {
            let ytod = self.years as f64 * 365.2425;
            let mtod = self.months as f64 * 30.437;
            (ytod + mtod + self.days as f64) as i32
        }
        pub fn from_number(number: &Number, range: &Range, errors: &mut Vec<Error>) -> Duration {
            let mut duration = Duration::new();
            match number.unit.id.as_str() {
                "days" => duration.days = number.to_double() as i32,
                "months" => duration.months = number.to_double() as i32,
                "years" => duration.years = number.to_double() as i32,
                _ => errors.push(Error::build(ErrorId::EExplicitUnitsExpected, range.clone(), &["days, months, years"])),
            }
            duration
        }
    }
}