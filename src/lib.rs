//! A Rust client for the [YNAB API](https://api.ynab.com).
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use rust_ynab::{Client, PlanId};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new(&std::env::var("YNAB_TOKEN")?)?;
//!     let plans = client.get_plans().include_accounts().send().await?;
//!     for plan in plans {
//!         println!("{}", plan.name);
//!     }
//!     Ok(())
//! }
//! ```

mod ynab;

pub use ynab::account::{Account, AccountType, SaveAccount};
pub use ynab::category::{
    Category, CategoryGroup, GoalType, NewCategory, SaveCategory, SaveCategoryGroup,
    SaveMonthCategory,
};
pub use ynab::client::Client;
pub use ynab::common::{CurrencyFormat, DateFormat};
pub use ynab::errors::{ApiError, Error};
pub use ynab::month::Month;
pub use ynab::movements::{MoneyMovement, MoneyMovementGroup};
pub use ynab::payee::{Payee, PayeeLocation, PostPayee, SavePayee};
pub use ynab::plan::{Plan, PlanId, PlanSettings};
pub use ynab::transaction::{
    ClearedStatus, ExistingTransaction, FlagColor, Frequency, NewTransaction,
    SaveScheduledTransaction, SaveSubTransaction, SaveTransactionResponse,
    SaveTransactionWithIdOrImportId, SaveTransactionsResponse, ScheduledSubtransaction,
    ScheduledTransaction, Subtransaction, Transaction, TransactionType,
};
pub use ynab::user::User;

/// Converts a YNAB milliunit amount to a dollar amount.
///
/// # Examples
///
/// ```
/// assert_eq!(rust_ynab::milliunits_to_amount(50000), 50.0);
/// assert_eq!(rust_ynab::milliunits_to_amount(-1500), -1.5);
/// ```
pub fn milliunits_to_amount(milliunits: i64) -> f64 {
    milliunits as f64 / 1000.0
}
