use std::ops::{Add, Sub, Mul, Div};
use crate::errors::{Error, ErrorId};
    use crate::number::Number;
use crate::tokenizer::cursor::Range;

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

    pub fn from_days(days: i32) -> Duration {
        let mut duration = Duration::new();
        duration.years = (days as f64 / 365.2425).trunc() as i32;
        duration.days = (days as f64 % 365.2425) as i32;
        duration.months = (duration.days as f64 / 30.437) as i32;
        duration.days = (duration.days as f64 % 30.437) as i32;
        duration
    }

    pub fn from_months(months: i32) -> Duration {
        let mut duration = Duration::new();
        duration.days = 0;
        duration.years = (months as f64 / 12.0) as i32;
        duration.months = (months as f64 % 12.0) as i32;
        duration
    }

    //assumes 1 month = 30 days
    pub fn normalize(&mut self) {
        if self.days >= 0 && self.days <= 31 &&
            self.months >= 0 && self.months <= 12 &&
            self.years >= 0 {
            return;
        }

        let dur_from_days = Duration::from_days(self.days);
        self.days = dur_from_days.days;
        self.months += dur_from_days.months;
        self.years += dur_from_days.years;

        let dur_from_months = Duration::from_months(self.months);
        self.months = dur_from_months.months;
        self.years += dur_from_months.years;

        if self.months < 0 && self.years > 0{
            self.years -= 1;
            self.months += 12;
        }

        if self.days < 0 && self.months > 0 {
            self.months -= 1;
            self.days += 30;
        }

    }

    pub fn to_days(&self) -> i32 {
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

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration {
            days: self.days + rhs.days,
            months: self.months + rhs.months,
            years: self.years + rhs.years,
        }
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration {
            days: self.days - rhs.days,
            months: self.months - rhs.months,
            years: self.years - rhs.years,
        }
    }
}

impl Mul<&Number> for Duration {
    type Output = Duration;

    fn mul(self, rhs: &Number) -> Self::Output {
        let d = rhs.to_double();
        Duration {
            days: (self.days as f64 * d) as i32,
            months: (self.months as f64 * d) as i32,
            years: (self.years as f64 * d) as i32,
        }
    }
}

impl Div<&Number> for Duration {
    type Output = Duration;

    fn div(self, rhs: &Number) -> Self::Output {
        let d = rhs.to_double();
        Duration {
            days: (self.days as f64 / d) as i32,
            months: (self.months as f64 / d) as i32,
            years: (self.years as f64 / d) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::date::Duration;

    #[test]
    fn test_duration() {
        let dur = Duration::from_days(-35);
        assert_eq!(dur.months, -1);
        let dur = Duration::from_days(-400);
        assert_eq!(dur.years, -1);
        assert_eq!(dur.months, -1);
        assert!(dur.days < 0);

        let mut dur = Duration::new();
        dur.days = -400;
        dur.normalize();
        assert_eq!(dur.years, -1);
        assert_eq!(dur.months, -1);
        assert!(dur.days < 0);


    }
}