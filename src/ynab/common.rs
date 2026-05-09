use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DateFormat {
    format: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrencyFormat {
    pub iso_code: String,
    example_format: String,
    decimal_digits: usize,
    decimal_separator: char,
    symbol_first: bool,
    group_separator: String,
    currency_symbol: String,
    display_symbol: bool,
}

pub const NO_PARAMS: Option<&[(&str, &str)]> = None;
