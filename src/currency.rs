use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub struct Currency {
    whole: i32,
    digits: u8,
}

impl Currency {
    fn zero() -> Self {
        Self {
            whole: 0,
            digits: 0,
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:02}", self.whole, self.digits)
    }
}

impl FromStr for Currency {
    type Err = ParseCurrencyError;

    fn from_str(from: &str) -> Result<Self, Self::Err> {
        if let Some((whole_chars, digit_chars)) = from.split_once(".") {
            let mut negative = false;
            let mut whole = 0i32;
            let mut digits = 0;
            let mut magnitude = 1i32;

            for c in whole_chars.chars().rev() {
                if c == '(' {
                    negative = true
                } else if c.is_numeric() {
                    if let Some(digit) = c.to_digit(10) {
                        whole += (digit as i32) * magnitude;
                        magnitude *= 10;
                    }
                }
            }

            magnitude = 1;
            for c in digit_chars.chars().rev() {
                if c.is_numeric() {
                    if let Some(digit) = c.to_digit(10) {
                        digits += (digit as i32) * magnitude;
                        magnitude *= 10;
                    }
                }
            }

            if negative {
                whole *= -1;
            }

            let result = Currency {
                whole,
                digits: digits as u8,
            };
            Ok(result)
        } else if from.replace(" ", "") == "$-" {
            Ok(Currency::zero())
        } else {
            Err(ParseCurrencyError::new(from))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseCurrencyError {
    source: String,
}

impl ParseCurrencyError {
    fn new(s: &str) -> Self {
        ParseCurrencyError {
            source: s.to_string(),
        }
    }
}

impl fmt::Display for ParseCurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not parse {} into a currency.", self.source)
    }
}

impl std::error::Error for ParseCurrencyError {
    fn description(&self) -> &str {
        "Failed to parse currency"
    }
}

pub fn deserialize_money<'de, D>(d: D) -> std::result::Result<Currency, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(d)?;
    Currency::from_str(&buf).map_err(serde::de::Error::custom)
}
