use serde::{Deserialize, Deserializer};
use std::fmt;

use chrono::prelude::*;

use crate::currency::{deserialize_money, Currency};

#[derive(Deserialize, Debug)]
pub struct CsvRecord {
    #[serde(rename = "ACCOUNT")]
    pub account: String,
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "Date", deserialize_with = "parse_date_time")]
    pub date: DateTime<FixedOffset>,
    #[serde(rename = "Amount", deserialize_with = "deserialize_money")]
    pub amount: Currency,
    #[serde(rename = "Balance", deserialize_with = "deserialize_money")]
    pub balance: Currency,
    #[serde(rename = "Vendor")]
    pub vendor: String,
    #[serde(rename = "Digits")]
    pub digits: Option<String>,
    #[serde(rename = "Type")]
    pub transaction_type: String,
    #[serde(rename = "Category")]
    pub category: Option<String>,
    #[serde(rename = "Subcategory")]
    pub subcategory: Option<String>,
    #[serde(rename = "Notes")]
    pub notes: Option<String>,
}

impl fmt::Display for CsvRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Account: {}", self.account)?;
        write!(f, ", Id: {}", self.id)?;
        write!(f, ", Date: {}", self.date)?;
        write!(f, ", Amount: {}", self.amount)?;
        write!(f, ", Balance: {}", self.balance)?;
        write!(f, ", Vendor: {}", self.vendor)?;
        write!(f, ", Digits: {:?}", self.digits)?;
        write!(f, ", Type: {}", self.transaction_type)?;
        write!(f, ", Category: {:?})", &self.category)?;
        write!(f, ", Subcategory: {:?})", &self.subcategory)?;
        write!(f, ", Notes: {:?})", &self.notes)
    }
}

pub fn parse_date_time<'de, D>(d: D) -> std::result::Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(d)?;

    let formatted = format!("{}  00:00:00 +00:00", buf);

    DateTime::parse_from_str(&formatted, "%m/%d/%Y %H:%M:%S %z").map_err(serde::de::Error::custom)
}
