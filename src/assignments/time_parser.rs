use chrono::TimeZone;
use chrono::{NaiveDateTime, Utc};
use chrono_tz::Europe::Helsinki;
use chrono_tz::Tz;
use core::fmt;

use chrono::DateTime;

type Result<T> = std::result::Result<T, TimeParseError>;

#[derive(Debug, PartialEq)]
pub enum TimeParseError {
    InvalidUnit(char),
    InvalidNumber(String),
    NoTime,
    TrailingDigits,
    EmptyInput,
}

// TODO show full string and error position like rustc does
impl fmt::Display for TimeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // using backticks, this is meant for discord
        const TIME_UNITS: &str = "`d`, `h`, `m` or `s`";

        match self {
            Self::InvalidUnit(c) => {
                write!(f, "invalid time unit `{c}`, expected one of {TIME_UNITS}")
            }
            Self::InvalidNumber(s) => write!(f, "invalid number `{s}`, expected a valid number"),
            Self::NoTime => write!(f, "no time provided"),
            Self::TrailingDigits => {
                write!(
                    f,
                    "trailing digits without a unit, specify one of {TIME_UNITS}"
                )
            }
            Self::EmptyInput => write!(f, "input is empty"),
        }
    }
}

fn multiplier(c: char) -> Result<u64> {
    match c {
        'd' => Ok(60 * 60 * 24),
        'h' => Ok(60 * 60),
        'm' => Ok(60),
        's' => Ok(1),
        _ => Err(TimeParseError::InvalidUnit(c)),
    }
}

pub fn parse_time(time: &str) -> Result<u64> {
    if time.is_empty() {
        return Err(TimeParseError::EmptyInput);
    }

    let mut chars = time.chars();

    let mut num_chars: Vec<char> = Vec::new();
    let mut num: u64 = 0;

    while let Some(c) = chars.next() {
        if c.is_numeric() {
            num_chars.push(c);
        } else {
            if num_chars.is_empty() {
                return Err(TimeParseError::NoTime);
            }
            let num_str = num_chars.iter().collect::<String>();
            num += num_str
                .parse::<u64>()
                .map_err(|_| TimeParseError::InvalidNumber(num_str))?
                * multiplier(c)?;
            num_chars.clear();
        }
    }

    if !num_chars.is_empty() {
        Err(TimeParseError::TrailingDigits)
    } else {
        Ok(num)
    }
}

pub fn parse_deadline(input: &str) -> Option<DateTime<Tz>> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let (date, time) = (*parts.get(0)?, *parts.get(1)?);

    let date_parts: Vec<&str> = date.split('.').collect();
    let time_parts: Vec<&str> = time.split('.').collect();

    let padded = format!(
        "{:0>2}.{:0>2}.{} {:0>2}.{:0>2}",
        date_parts.get(0)?,
        date_parts.get(1)?,
        date_parts.get(2)?,
        time_parts.get(0)?,
        time_parts.get(1)?
    );

    let parsed = NaiveDateTime::parse_from_str(&padded, "%d.%m.%Y %H.%M").expect(".igm");
    let dt = Helsinki
        .from_local_datetime(&parsed)
        .single()
        .expect("ufkcyou");

    Some(dt)
}

pub mod tests {
    use super::*;
    use TimeParseError::*;

    #[test]
    fn valid_time_amounts() {
        assert_eq!(Ok(1), parse_time("1s"));
        assert_eq!(Ok(60), parse_time("1m"));
        assert_eq!(Ok(3600), parse_time("1h"));
        assert_eq!(Ok(86400), parse_time("1d"));

        assert_eq!(Ok(86400), parse_time("1440m"));
        assert_eq!(Ok(86400 * 2), parse_time("2d"));

        assert_eq!(Ok(4), parse_time("1s1s1s1s"));
    }

    #[test]
    fn time_errors() {
        assert_eq!(Err(NoTime), parse_time("d"));
        assert_eq!(Err(NoTime), parse_time("1dd"));
        assert_eq!(Err(EmptyInput), parse_time(""));
        assert_eq!(Err(InvalidUnit('c')), parse_time("1c"));
        assert_eq!(Err(TrailingDigits), parse_time("1d2"));
    }
}
