use actix_settings::{Error, Parse};
use once_cell::unsync::Lazy;
use regex::Regex;
use serde::de;
use std::fmt;
use std::ops::Deref;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct DurationValue(Duration);

impl From<Duration> for DurationValue {
    fn from(duration: Duration) -> Self {
        DurationValue(duration)
    }
}

impl Deref for DurationValue {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for DurationValue {
    fn parse(string: &str) -> Result<Self, Error> {
        const DURATION_FMT: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(?<digits>\d+)\s*(?<units>seconds?|minutes?|hours?|days?|)$")
                .expect("Failed to compile regex for Duration")
        });

        macro_rules! invalid_value {
            ($got:expr) => {
                Err(Error::InvalidValue {
                    expected: "a string of the format \"N seconds\", \"N minutes\", \"N hours\" or \"N days\" where N is an integer > 0",
                    got: $got.to_string(),
            file: file!(),
            line: line!(),
            column: column!(),
                })
            }
        }

        match DURATION_FMT.captures(string) {
            Some(caps) => {
                let (_, [digits, units]) = caps.extract();
                let mul = match units {
                    "seconds" | "second" | "" => 1,
                    "minutes" | "minute" => 60,
                    "hours" | "hour" => 3600,
                    "days" | "day" => 3600 * 24,
                    _ => invalid_value!(string)?,
                };
                let digits_as_int: u64 = digits.parse()?;
                Ok(Duration::from_secs(digits_as_int * mul).into())
            }
            None => invalid_value!(string),
        }
    }
}

impl<'de> de::Deserialize<'de> for DurationValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct DurationValueVisitor;

        impl<'de> de::Visitor<'de> for DurationValueVisitor {
            type Value = DurationValue;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let msg = "a string of the format \"N seconds\", \"N minutes\", \"N hours\" or \"N days\" where N is an integer > 0";
                f.write_str(msg)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match DurationValue::parse(value) {
                    Ok(value) => Ok(value),
                    Err(Error::InvalidValue { expected, got, .. }) => Err(
                        de::Error::invalid_value(de::Unexpected::Str(&got), &expected),
                    ),
                    Err(_) => unreachable!(),
                }
            }
        }

        deserializer.deserialize_string(DurationValueVisitor)
    }
}
