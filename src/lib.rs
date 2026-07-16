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

#[cfg(feature = "polars")]
pub use ynab::polars::{IntoDataFrame, account_debt_history};

#[cfg(feature = "oauth")]
pub use ynab::oauth::{OAuthConfig, OAuthTokens};

pub use ynab::account::{Account, AccountType, SaveAccount, SaveAccountType};
pub use ynab::category::{
    Category, CategoryGroup, GoalType, NewCategory, SaveCategory, SaveCategoryGroup,
    SaveMonthCategory,
};
pub use ynab::client::Client;
pub use ynab::common::{CurrencyFormat, DateFormat, ServerKnowledge};
pub use ynab::errors::{ApiError, Error};
pub use ynab::month::Month;
pub use ynab::movements::{MoneyMovement, MoneyMovementGroup};
pub use ynab::payee::{Payee, PayeeLocation, PostPayee, SavePayee};
pub use ynab::plan::{Plan, PlanDetails, PlanId, PlanSettings};
pub use ynab::transaction::{
    ClearedStatus, ExistingTransaction, FlagColor, Frequency, GetHybridTransactionsBuilder,
    HybridTransaction, HybridTransactionType, NewTransaction, SaveScheduledTransaction,
    SaveSubTransaction, SaveTransactionResponse, SaveTransactionWithIdOrImportId,
    SaveTransactionsResponse, ScheduledSubtransaction, ScheduledTransaction,
    ScheduledTransactionSummary, Subtransaction, Transaction, TransactionSummary, TransactionType,
};
pub use ynab::user::User;

/// Converts a YNAB milliunit amount to a dollar amount.
/// For display only. For arithmetic use milliunits.
/// # Examples
///
/// ```
/// assert_eq!(rust_ynab::milliunits_to_amount(50000), 50.0);
/// assert_eq!(rust_ynab::milliunits_to_amount(-1500), -1.5);
/// ```
pub fn milliunits_to_amount(milliunits: i64) -> f64 {
    milliunits as f64 / 1000.0
}
