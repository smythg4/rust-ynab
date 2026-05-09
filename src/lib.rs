pub mod display;
pub mod ynab;

pub use ynab::account::Account;
pub use ynab::category::{Category, CategoryGroup};
pub use ynab::client::Client;
pub use ynab::common::{CurrencyFormat, DateFormat};
pub use ynab::errors::{ApiError, Error};
pub use ynab::month::Month;
pub use ynab::movements::{MoneyMovement, MoneyMovementGroup};
pub use ynab::payee::{Payee, PayeeLocation};
pub use ynab::plan::{Plan, PlanId, PlanSettings};
pub use ynab::transaction::{
    ClearedStatus, FlagColor, Frequency, ScheduledSubtransaction, ScheduledTransaction,
    Subtransaction, Transaction, TransactionType,
};
pub use ynab::user::User;

/// Converts a YNAB milliunit amount to a dollar amount.
pub fn milliunits_to_amount(milliunits: i64) -> f64 {
    milliunits as f64 / 1000.0
}
