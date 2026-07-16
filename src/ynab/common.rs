use serde::{Deserialize, Serialize};

/// An opaque, monotonically increasing sync cursor returned alongside most list/write
/// responses. Store it and pass it back into `.with_server_knowledge()` on a later call to
/// receive only the changes since that point (a delta request).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct ServerKnowledge(pub i64);

impl std::fmt::Display for ServerKnowledge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for ServerKnowledge {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<ServerKnowledge> for i64 {
    fn from(value: ServerKnowledge) -> Self {
        value.0
    }
}

impl PartialEq<i64> for ServerKnowledge {
    fn eq(&self, other: &i64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<ServerKnowledge> for i64 {
    fn eq(&self, other: &ServerKnowledge) -> bool {
        *self == other.0
    }
}

impl PartialOrd<i64> for ServerKnowledge {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<ServerKnowledge> for i64 {
    fn partial_cmp(&self, other: &ServerKnowledge) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DateFormat {
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CurrencyFormat {
    pub iso_code: String,
    pub example_format: String,
    pub decimal_digits: usize,
    pub decimal_separator: String,
    pub symbol_first: bool,
    pub group_separator: String,
    pub currency_symbol: String,
    pub display_symbol: bool,
}

pub(crate) const NO_PARAMS: Option<&[(&str, &str)]> = None;
