use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DateFormat {
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CurrencyFormat {
    pub iso_code: String,
    pub example_format: String,
    pub decimal_digits: usize,
    pub decimal_separator: char,
    pub symbol_first: bool,
    pub group_separator: String,
    pub currency_symbol: String,
    pub display_symbol: bool,
}

pub(crate) const NO_PARAMS: Option<&[(&str, &str)]> = None;
