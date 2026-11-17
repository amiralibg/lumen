//! A small civil date type for the calendar and date picker (proleptic
//! Gregorian, no time zones).

use std::fmt;

/// A calendar date.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    /// 1..=12
    pub month: u8,
    /// 1..=31
    pub day: u8,
}

pub const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

impl Date {
    /// A date, with the day clamped to the month's length.
    pub fn new(year: i32, month: u8, day: u8) -> Date {
        let month = month.clamp(1, 12);
        Date { year, month, day: day.clamp(1, days_in_month(year, month)) }
    }

    /// Today in UTC (the local date is close enough for UI defaults).
    pub fn today() -> Date {
        let secs = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        Date::from_days((secs / 86_400) as i64)
    }

    /// Days since 1970-01-01.
    pub fn to_days(self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let y = self.year as i64 - if self.month <= 2 { 1 } else { 0 };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    pub fn from_days(z: i64) -> Date {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let y = (yoe + era * 400 + if m <= 2 { 1 } else { 0 }) as i32;
        Date { year: y, month: m, day: d }
    }

    /// 0 = Monday .. 6 = Sunday.
    pub fn weekday(self) -> u8 {
        ((self.to_days() + 3).rem_euclid(7)) as u8
    }

    pub fn add_days(self, n: i64) -> Date {
        Date::from_days(self.to_days() + n)
    }

    /// Same day `n` months later, clamped to the month's length.
    pub fn add_months(self, n: i32) -> Date {
        let total = self.year * 12 + (self.month as i32 - 1) + n;
        let (y, m) = (total.div_euclid(12), (total.rem_euclid(12) + 1) as u8);
        Date::new(y, m, self.day)
    }

    pub fn first_of_month(self) -> Date {
        Date { day: 1, ..self }
    }

    pub fn month_name(self) -> &'static str {
        MONTHS[self.month as usize - 1]
    }

    /// "Nov 17, 2026".
    pub fn short(self) -> String {
        format!("{} {}, {}", &self.month_name()[..3], self.day, self.year)
    }
}

impl fmt::Display for Date {
    /// ISO 8601: `2026-11-17`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl std::str::FromStr for Date {
    type Err = ();
    fn from_str(s: &str) -> Result<Date, ()> {
        let mut it = s.trim().split('-');
        let y = it.next().ok_or(())?.parse().map_err(|_| ())?;
        let m: u8 = it.next().ok_or(())?.parse().map_err(|_| ())?;
        let d: u8 = it.next().ok_or(())?.parse().map_err(|_| ())?;
        if !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
            return Err(());
        }
        Ok(Date { year: y, month: m, day: d })
    }
}

pub fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(y: i32, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_weekday() {
        for z in [-800_000i64, -1, 0, 1, 20_000, 20_774, 100_000] {
            assert_eq!(Date::from_days(z).to_days(), z);
        }
        assert_eq!(Date::from_days(0), Date::new(1970, 1, 1));
        // 1970-01-01 was a Thursday.
        assert_eq!(Date::new(1970, 1, 1).weekday(), 3);
        assert_eq!(Date::new(2026, 11, 17).weekday(), 1, "Tuesday");
    }

    #[test]
    fn months_clamp() {
        assert_eq!(Date::new(2024, 1, 31).add_months(1), Date::new(2024, 2, 29));
        assert_eq!(Date::new(2025, 12, 5).add_months(1), Date::new(2026, 1, 5));
        assert_eq!(Date::new(2025, 1, 5).add_months(-1), Date::new(2024, 12, 5));
        assert_eq!("2026-02-29".parse::<Date>(), Err(()));
        assert_eq!("2028-02-29".parse::<Date>().unwrap().to_string(), "2028-02-29");
    }
}
