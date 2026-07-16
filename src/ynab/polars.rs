//! Polars DataFrame conversions for YNAB types.
//!
//! Enable with the `polars` feature flag:
//!
//! ```toml
//! [dependencies]
//! rust-ynab = { version = "...", features = ["polars"] }
//! ```
//!
//! # Type mapping
//!
//! | Rust type | Polars dtype |
//! |---|---|
//! | `bool` | `Boolean` |
//! | `i64` | `Int64` |
//! | `String` | `String` |
//! | `Option<T>` | nullable column of T's dtype |
//! | `Uuid` | `String` |
//! | `NaiveDate` | `Date` (days since Unix epoch) |
//! | `DateTime<Utc>` | `Datetime(Milliseconds)` |
//! | `enum` | `String` (Debug representation) |
//!
//! # Nested types
//!
//! Types that contain nested collections (e.g. `Plan::accounts`,
//! `CategoryGroup::categories`, `Transaction::subtransactions`) are not
//! inlined. Instead a `*_count` column is emitted and the nested type has
//! its own `IntoDataFrame` impl that can be joined on a shared ID column.

use crate::{
    Account, Category, CategoryGroup, HybridTransaction, MoneyMovement, MoneyMovementGroup, Month,
    Payee, PayeeLocation, Plan, ScheduledSubtransaction, ScheduledTransaction,
    ScheduledTransactionSummary, Subtransaction, Transaction, TransactionSummary,
};
use chrono::NaiveDate;
use polars::prelude::*;

/// Converts a YNAB type into a Polars [`DataFrame`].
///
/// Implemented for all major YNAB collection types. Call [`into_dataframe`](IntoDataFrame::into_dataframe)
/// on any `Vec<T>` where `T` is a YNAB API type.
///
/// # Examples
///
/// ```no_run
/// use rust_ynab::{Client, PlanId};
/// use rust_ynab::IntoDataFrame;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new(&std::env::var("YNAB_TOKEN")?)?;
/// let (accounts, _) = client.get_accounts(PlanId::LastUsed).send().await?;
/// let df = accounts.into_dataframe();
/// println!("{df}");
/// # Ok(())
/// # }
/// ```
pub trait IntoDataFrame {
    /// Consumes `self` and returns a [`DataFrame`] with one column per field.
    fn into_dataframe(self) -> DataFrame;
}

fn epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
}

/// Generates an `IntoDataFrame` impl for `Vec<$Type>`. Each column is `"name": field = expr`,
/// where `expr` is evaluated once per item (bound as `$item`) to produce that column's value.
/// Add `, cast = DataType::X` to a column to `.cast()` the assembled `Series` (used for
/// `Date`/`Datetime` columns, which can't be pushed as their target dtype directly).
///
/// Not used for `Plan`, whose `currency_format` fans one `Option` field out into 8 columns —
/// a genuinely different shape from every other type here, left hand-written rather than
/// contorting this macro to cover a one-off case.
macro_rules! into_dataframe {
    (
        $(#[$doc:meta])*
        $Type:ty, |$item:ident| {
            $( $name:literal : $field:ident = $push:expr $(, cast = $cast:expr)? );+ $(;)?
        }
    ) => {
        $(#[$doc])*
        impl IntoDataFrame for Vec<$Type> {
            fn into_dataframe(self) -> DataFrame {
                let size = self.len();
                $( let mut $field = Vec::with_capacity(size); )+
                for $item in self {
                    $( $field.push($push); )+
                }
                DataFrame::new(
                    size,
                    vec![
                        $( into_dataframe!(@col $name, $field $(, $cast)?) ),+
                    ],
                )
                .expect("all columns must have equal length")
            }
        }
    };
    (@col $name:literal, $field:ident) => {
        Series::new($name.into(), $field).into()
    };
    (@col $name:literal, $field:ident, $cast:expr) => {
        Series::new($name.into(), $field)
            .cast(&$cast)
            .unwrap()
            .into()
    };
}

into_dataframe!(
    /// `type` is stringified. `last_reconciled_at` is a `Datetime(Milliseconds)` column.
    /// `debt_interest_rates`, `debt_minimum_payments`, and `debt_escrow_amounts` are keyed by a
    /// variable number of dates per account, so they don't fit a flat column — only their entry
    /// counts appear here. Use [`account_debt_history`] for the actual dated values, joined on
    /// `account_id`.
    Account, |a| {
    "id": id = a.id.to_string();
    "name": name = a.name;
    "type": acct_type = format!("{:?}", a.acct_type);
    "on_budget": on_budget = a.on_budget;
    "closed": closed = a.closed;
    "note": note = a.note;
    "balance": balance = a.balance;
    "cleared_balance": cleared_balance = a.cleared_balance;
    "uncleared_balance": uncleared_balance = a.uncleared_balance;
    "transfer_payee_id": transfer_payee_id = a.transfer_payee_id.map(|tpi| tpi.to_string());
    "direct_import_linked": direct_import_linked = a.direct_import_linked;
    "direct_import_in_error": direct_import_in_error = a.direct_import_in_error;
    "last_reconciled_at": last_reconciled_at = a.last_reconciled_at.map(|lra| lra.timestamp_millis()), cast = DataType::Datetime(TimeUnit::Milliseconds, None);
    "debt_original_balance": debt_original_balance = a.debt_original_balance;
    "debt_interest_rates_count": debt_interest_rates_count = a.debt_interest_rates.as_ref().map(|m| m.len() as u32).unwrap_or(0);
    "debt_minimum_payments_count": debt_minimum_payments_count = a.debt_minimum_payments.as_ref().map(|m| m.len() as u32).unwrap_or(0);
    "debt_escrow_amounts_count": debt_escrow_amounts_count = a.debt_escrow_amounts.as_ref().map(|m| m.len() as u32).unwrap_or(0);
    "deleted": deleted = a.deleted;
});

/// Flattens `Account::debt_interest_rates`/`debt_minimum_payments`/`debt_escrow_amounts` into a
/// long-format table — one row per (account, kind, dated entry) — since each account can carry a
/// different number of these across different dates, which doesn't fit a flat column on
/// [`Vec<Account>::into_dataframe`](IntoDataFrame::into_dataframe). Join to it on `account_id`.
/// `month` is the raw date-string key as returned by the API (e.g. `"2024-01-01"`), left as a
/// `String` rather than parsed, since it isn't a value pushed per-account like every other date
/// column here — cast it yourself (`pl.col("month").str.to_date()`) if you need it typed.
pub fn account_debt_history(accounts: &[Account]) -> DataFrame {
    let mut account_id = Vec::new();
    let mut kind = Vec::new();
    let mut month = Vec::new();
    let mut amount = Vec::new();
    for a in accounts {
        let maps: [(&str, &Option<std::collections::HashMap<String, i64>>); 3] = [
            ("interest_rate", &a.debt_interest_rates),
            ("minimum_payment", &a.debt_minimum_payments),
            ("escrow_amount", &a.debt_escrow_amounts),
        ];
        for (label, map) in maps {
            let Some(map) = map else { continue };
            for (date_str, value) in map {
                account_id.push(a.id.to_string());
                kind.push(label);
                month.push(date_str.clone());
                amount.push(*value);
            }
        }
    }
    let size = account_id.len();
    DataFrame::new(
        size,
        vec![
            Series::new("account_id".into(), account_id).into(),
            Series::new("kind".into(), kind).into(),
            Series::new("month".into(), month).into(),
            Series::new("amount".into(), amount).into(),
        ],
    )
    .expect("all columns must have equal length")
}

into_dataframe!(
    /// `date` is a `Date` column. `cleared`, `flag_color`, and `debt_transaction_type` are stringified.
    /// `subtransactions` is dropped — use `Vec<Subtransaction>::into_dataframe()` and join on `transaction_id`.
    Transaction, |t| {
    "id": id = t.id;
    "account_name": account_name = t.account_name;
    "date": date = (t.date - epoch()).num_days() as i32, cast = DataType::Date;
    "amount": amount = t.amount;
    "memo": memo = t.memo;
    "cleared": cleared = format!("{:?}", t.cleared);
    "approved": approved = t.approved;
    "flag_color": flag_color = t.flag_color.map(|f| format!("{:?}", f));
    "flag_name": flag_name = t.flag_name;
    "account_id": account_id = t.account_id.to_string();
    "payee_id": payee_id = t.payee_id.map(|u| u.to_string());
    "payee_name": payee_name = t.payee_name;
    "category_id": category_id = t.category_id.map(|u| u.to_string());
    "category_name": category_name = t.category_name;
    "matched_transaction_id": matched_transaction_id = t.matched_transaction_id;
    "import_id": import_id = t.import_id;
    "import_payee_name": import_payee_name = t.import_payee_name;
    "import_payee_name_original": import_payee_name_original = t.import_payee_name_original;
    "debt_transaction_type": debt_transaction_type = t.debt_transaction_type.map(|d| format!("{:?}", d));
    "deleted": deleted = t.deleted;
});

into_dataframe!(
    /// Returned by `get_transactions_by_category`/`get_transactions_by_payee`. `type` (renamed
    /// from `ttype`), `date`, `cleared`, `flag_color`, and `debt_transaction_type` follow the
    /// same conventions as `Vec<Transaction>`. `parent_transaction_id` links a `"subtransaction"`
    /// row back to its parent — use it to distinguish real transactions from split line items,
    /// which the plain `Transaction`/`TransactionSummary` shapes can't represent.
    HybridTransaction, |h| {
    "type": ttype = format!("{:?}", h.ttype);
    "id": id = h.id;
    "date": date = (h.date - epoch()).num_days() as i32, cast = DataType::Date;
    "amount": amount = h.amount;
    "memo": memo = h.memo;
    "cleared": cleared = format!("{:?}", h.cleared);
    "approved": approved = h.approved;
    "account_id": account_id = h.account_id.to_string();
    "account_name": account_name = h.account_name;
    "category_name": category_name = h.category_name;
    "parent_transaction_id": parent_transaction_id = h.parent_transaction_id;
    "flag_color": flag_color = h.flag_color.map(|f| format!("{:?}", f));
    "flag_name": flag_name = h.flag_name;
    "payee_id": payee_id = h.payee_id.map(|u| u.to_string());
    "payee_name": payee_name = h.payee_name;
    "category_id": category_id = h.category_id.map(|u| u.to_string());
    "matched_transaction_id": matched_transaction_id = h.matched_transaction_id;
    "import_id": import_id = h.import_id;
    "import_payee_name": import_payee_name = h.import_payee_name;
    "import_payee_name_original": import_payee_name_original = h.import_payee_name_original;
    "transfer_account_id": transfer_account_id = h.transfer_account_id.map(|u| u.to_string());
    "transfer_transaction_id": transfer_transaction_id = h.transfer_transaction_id;
    "debt_transaction_type": debt_transaction_type = h.debt_transaction_type.map(|d| format!("{:?}", d));
    "deleted": deleted = h.deleted;
});

into_dataframe!(
    /// The transaction shape used in the plan export (`PlanDetails.transactions`) — a reduced
    /// form of `Transaction` with no `account_name`, `payee_name`, or `category_name`. Join to
    /// `Vec<Account>`/`Vec<Category>`/`Vec<Payee>`::into_dataframe()` on the respective id columns
    /// for those. `debt_transaction_type` is stringified like the other enum columns here.
    TransactionSummary, |t| {
    "id": id = t.id;
    "date": date = (t.date - epoch()).num_days() as i32, cast = DataType::Date;
    "amount": amount = t.amount;
    "memo": memo = t.memo;
    "cleared": cleared = format!("{:?}", t.cleared);
    "approved": approved = t.approved;
    "flag_color": flag_color = t.flag_color.map(|f| format!("{:?}", f));
    "flag_name": flag_name = t.flag_name;
    "account_id": account_id = t.account_id.to_string();
    "payee_id": payee_id = t.payee_id.map(|u| u.to_string());
    "category_id": category_id = t.category_id.map(|u| u.to_string());
    "matched_transaction_id": matched_transaction_id = t.matched_transaction_id;
    "import_id": import_id = t.import_id;
    "import_payee_name": import_payee_name = t.import_payee_name;
    "import_payee_name_original": import_payee_name_original = t.import_payee_name_original;
    "transfer_account_id": transfer_account_id = t.transfer_account_id.map(|u| u.to_string());
    "transfer_transaction_id": transfer_transaction_id = t.transfer_transaction_id;
    "debt_transaction_type": debt_transaction_type = t.debt_transaction_type.map(|d| format!("{:?}", d));
    "deleted": deleted = t.deleted;
});

into_dataframe!(
    /// `categories` is dropped — use `Vec<Category>::into_dataframe()` and join on `category_group_id`.
    /// A `category_count` column is included as a convenience.
    CategoryGroup, |g| {
    "id": id = g.id.to_string();
    "name": name = g.name;
    "hidden": hidden = g.hidden;
    "deleted": deleted = g.deleted;
    "category_count": category_count = g.categories.len() as u32;
});

into_dataframe!(
    /// All goal date fields are `Date` columns. `goal_snoozed_at` is `Datetime(Milliseconds)`.
    /// `goal_type` is stringified. `usize` goal fields are cast to `i32`.
    Category, |c| {
    "id": id = c.id.to_string();
    "category_group_id": category_group_id = c.category_group_id.to_string();
    "category_group_name": category_group_name = c.category_group_name;
    "name": name = c.name;
    "hidden": hidden = c.hidden;
    "original_category_group_id": original_category_group_id = c.original_category_group_id.map(|u| u.to_string());
    "note": note = c.note;
    "budgeted": budgeted = c.budgeted;
    "activity": activity = c.activity;
    "balance": balance = c.balance;
    "goal_type": goal_type = c.goal_type.map(|g| format!("{:?}", g));
    "goal_needs_whole_amount": goal_needs_whole_amount = c.goal_needs_whole_amount;
    "goal_day": goal_day = c.goal_day.map(|v| v as i32);
    "goal_cadence": goal_cadence = c.goal_cadence.map(|v| v as i32);
    "goal_cadence_frequency": goal_cadence_frequency = c.goal_cadence_frequency.map(|v| v as i32);
    "goal_creation_month": goal_creation_month = c.goal_creation_month.map(|d| (d - epoch()).num_days() as i32), cast = DataType::Date;
    "goal_target": goal_target = c.goal_target;
    "goal_target_date": goal_target_date = c.goal_target_date.map(|d| (d - epoch()).num_days() as i32), cast = DataType::Date;
    "goal_target_month": goal_target_month = c.goal_target_month.map(|d| (d - epoch()).num_days() as i32), cast = DataType::Date;
    "goal_percentage_complete": goal_percentage_complete = c.goal_percentage_complete.map(|v| v as i32);
    "goal_months_to_budget": goal_months_to_budget = c.goal_months_to_budget.map(|v| v as i32);
    "goal_under_funded": goal_under_funded = c.goal_under_funded;
    "goal_overall_funded": goal_overall_funded = c.goal_overall_funded;
    "goal_overall_left": goal_overall_left = c.goal_overall_left;
    "goal_snoozed_at": goal_snoozed_at = c.goal_snoozed_at.map(|dt| dt.timestamp_millis()), cast = DataType::Datetime(TimeUnit::Milliseconds, None);
    "deleted": deleted = c.deleted;
});

into_dataframe!(
    /// `month` is a `Date` column. `categories` is dropped — use `Vec<Category>::into_dataframe()`.
    /// A `category_count` column is included as a convenience.
    Month, |m| {
    "month": month = (m.month - epoch()).num_days() as i32, cast = DataType::Date;
    "note": note = m.note;
    "income": income = m.income;
    "budgeted": budgeted = m.budgeted;
    "activity": activity = m.activity;
    "to_be_budgeted": to_be_budgeted = m.to_be_budgeted;
    "age_of_money": age_of_money = m.age_of_money.map(|v| v as i32);
    "deleted": deleted = m.deleted;
    "category_count": category_count = m.categories.len() as u32;
});

into_dataframe!(
    /// `month` is a nullable `Date` column. `moved_at` is a nullable `Datetime(Milliseconds)` column.
    MoneyMovement, |m| {
    "id": id = m.id.to_string();
    "month": month = m.month.map(|d| (d - epoch()).num_days() as i32), cast = DataType::Date;
    "moved_at": moved_at = m.moved_at.map(|dt| dt.timestamp_millis()), cast = DataType::Datetime(TimeUnit::Milliseconds, None);
    "note": note = m.note;
    "money_movement_group_id": money_movement_group_id = m.money_movement_group_id.map(|u| u.to_string());
    "performed_by_user_id": performed_by_user_id = m.performed_by_user_id.map(|u| u.to_string());
    "from_category_id": from_category_id = m.from_category_id.map(|u| u.to_string());
    "to_category_id": to_category_id = m.to_category_id.map(|u| u.to_string());
    "amount": amount = m.amount;
});

into_dataframe!(
    /// `group_created_at` is a `Datetime(Milliseconds)` column. `month` is a `Date` column.
    MoneyMovementGroup, |g| {
    "id": id = g.id.to_string();
    "group_created_at": group_created_at = g.group_created_at.timestamp_millis(), cast = DataType::Datetime(TimeUnit::Milliseconds, None);
    "month": month = (g.month - epoch()).num_days() as i32, cast = DataType::Date;
    "note": note = g.note;
    "performed_by_user_id": performed_by_user_id = g.performed_by_user_id.map(|u| u.to_string());
});

into_dataframe!(
    /// `transfer_account_id` is a nullable `String` column.
    Payee, |p| {
    "id": id = p.id.to_string();
    "name": name = p.name;
    "transfer_account_id": transfer_account_id = p.transfer_account_id.map(|u| u.to_string());
    "deleted": deleted = p.deleted;
});

into_dataframe!(
    /// `latitude` and `longitude` are `String` columns as returned by the API.
    PayeeLocation, |p| {
    "id": id = p.id.to_string();
    "payee_id": payee_id = p.payee_id.to_string();
    "latitude": latitude = p.latitude;
    "longitude": longitude = p.longitude;
    "deleted": deleted = p.deleted;
});

/// `DateFormat` and `CurrencyFormat` are flattened into individual, nullable columns prefixed
/// with `date_format` and `currency_` (both are `Option` on `Plan` — YNAB may not have a format
/// available for a plan). `accounts` is dropped — an `account_count` column is included.
/// `first_month` and `last_month` are `Date` columns. `last_modified_on` is `Datetime(Milliseconds)`.
impl IntoDataFrame for Vec<Plan> {
    fn into_dataframe(self) -> DataFrame {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut name = Vec::with_capacity(size);
        let mut last_modified_on = Vec::with_capacity(size);
        let mut first_month = Vec::with_capacity(size);
        let mut last_month = Vec::with_capacity(size);
        let mut date_format = Vec::with_capacity(size);
        let mut currency_iso_code = Vec::with_capacity(size);
        let mut currency_example_format = Vec::with_capacity(size);
        let mut currency_decimal_digits = Vec::with_capacity(size);
        let mut currency_decimal_separator = Vec::with_capacity(size);
        let mut currency_symbol_first = Vec::with_capacity(size);
        let mut currency_group_separator = Vec::with_capacity(size);
        let mut currency_symbol = Vec::with_capacity(size);
        let mut currency_display_symbol = Vec::with_capacity(size);
        let mut account_count = Vec::with_capacity(size);
        for p in self {
            id.push(p.id.to_string());
            name.push(p.name);
            last_modified_on.push(p.last_modified_on.timestamp_millis());
            first_month.push((p.first_month - epoch).num_days() as i32);
            last_month.push((p.last_month - epoch).num_days() as i32);
            date_format.push(p.date_format.map(|d| d.format));
            match p.currency_format {
                Some(cf) => {
                    currency_iso_code.push(Some(cf.iso_code));
                    currency_example_format.push(Some(cf.example_format));
                    currency_decimal_digits.push(Some(cf.decimal_digits as i32));
                    currency_decimal_separator.push(Some(cf.decimal_separator));
                    currency_symbol_first.push(Some(cf.symbol_first));
                    currency_group_separator.push(Some(cf.group_separator));
                    currency_symbol.push(Some(cf.currency_symbol));
                    currency_display_symbol.push(Some(cf.display_symbol));
                }
                None => {
                    currency_iso_code.push(None);
                    currency_example_format.push(None);
                    currency_decimal_digits.push(None);
                    currency_decimal_separator.push(None);
                    currency_symbol_first.push(None);
                    currency_group_separator.push(None);
                    currency_symbol.push(None);
                    currency_display_symbol.push(None);
                }
            }
            account_count.push(p.accounts.len() as u32);
        }
        let last_modified_on = Series::new("last_modified_on".into(), last_modified_on)
            .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
            .unwrap();
        let first_month = Series::new("first_month".into(), first_month)
            .cast(&DataType::Date)
            .unwrap();
        let last_month = Series::new("last_month".into(), last_month)
            .cast(&DataType::Date)
            .unwrap();
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                Series::new("name".into(), name).into(),
                last_modified_on.into(),
                first_month.into(),
                last_month.into(),
                Series::new("date_format".into(), date_format).into(),
                Series::new("currency_iso_code".into(), currency_iso_code).into(),
                Series::new("currency_example_format".into(), currency_example_format).into(),
                Series::new("currency_decimal_digits".into(), currency_decimal_digits).into(),
                Series::new(
                    "currency_decimal_separator".into(),
                    currency_decimal_separator,
                )
                .into(),
                Series::new("currency_symbol_first".into(), currency_symbol_first).into(),
                Series::new("currency_group_separator".into(), currency_group_separator).into(),
                Series::new("currency_symbol".into(), currency_symbol).into(),
                Series::new("currency_display_symbol".into(), currency_display_symbol).into(),
                Series::new("account_count".into(), account_count).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

into_dataframe!(
    /// `date_first` and `date_next` are `Date` columns. `frequency` and `flag_color` are stringified.
    /// `subtransactions` is dropped — use `Vec<ScheduledSubtransaction>::into_dataframe()` and join on `scheduled_transaction_id`.
    /// A `subtransaction_count` column is included as a convenience.
    ScheduledTransaction, |t| {
    "id": id = t.id.to_string();
    "date_first": date_first = (t.date_first - epoch()).num_days() as i32, cast = DataType::Date;
    "date_next": date_next = (t.date_next - epoch()).num_days() as i32, cast = DataType::Date;
    "frequency": frequency = format!("{:?}", t.frequency);
    "amount": amount = t.amount;
    "memo": memo = t.memo;
    "flag_color": flag_color = t.flag_color.map(|f| format!("{:?}", f));
    "flag_name": flag_name = t.flag_name;
    "account_id": account_id = t.account_id.to_string();
    "account_name": account_name = t.account_name;
    "payee_id": payee_id = t.payee_id.map(|u| u.to_string());
    "payee_name": payee_name = t.payee_name;
    "category_id": category_id = t.category_id.map(|u| u.to_string());
    "category_name": category_name = t.category_name;
    "transfer_account_id": transfer_account_id = t.transfer_account_id.map(|u| u.to_string());
    "subtransaction_count": subtransaction_count = t.subtransactions.len() as u32;
    "deleted": deleted = t.deleted;
});

into_dataframe!(
    /// The scheduled-transaction shape used in the plan export (`PlanDetails.scheduled_transactions`)
    /// — a reduced form of `ScheduledTransaction` with no `account_name`, `payee_name`, or
    /// `category_name`. Join to `Vec<Account>`/`Vec<Category>`/`Vec<Payee>::into_dataframe()` on
    /// the respective id columns for those.
    ScheduledTransactionSummary, |t| {
    "id": id = t.id.to_string();
    "date_first": date_first = (t.date_first - epoch()).num_days() as i32, cast = DataType::Date;
    "date_next": date_next = (t.date_next - epoch()).num_days() as i32, cast = DataType::Date;
    "frequency": frequency = format!("{:?}", t.frequency);
    "amount": amount = t.amount;
    "memo": memo = t.memo;
    "flag_color": flag_color = t.flag_color.map(|f| format!("{:?}", f));
    "flag_name": flag_name = t.flag_name;
    "account_id": account_id = t.account_id.to_string();
    "payee_id": payee_id = t.payee_id.map(|u| u.to_string());
    "category_id": category_id = t.category_id.map(|u| u.to_string());
    "transfer_account_id": transfer_account_id = t.transfer_account_id.map(|u| u.to_string());
    "deleted": deleted = t.deleted;
});

into_dataframe!(
    /// Join to `Vec<ScheduledTransaction>::into_dataframe()` on `scheduled_transaction_id`.
    ScheduledSubtransaction, |t| {
    "id": id = t.id.to_string();
    "scheduled_transaction_id": scheduled_transaction_id = t.scheduled_transaction_id.to_string();
    "amount": amount = t.amount;
    "memo": memo = t.memo;
    "payee_id": payee_id = t.payee_id.map(|u| u.to_string());
    "payee_name": payee_name = t.payee_name;
    "category_id": category_id = t.category_id.map(|u| u.to_string());
    "category_name": category_name = t.category_name;
    "transfer_account_id": transfer_account_id = t.transfer_account_id.map(|u| u.to_string());
    "deleted": deleted = t.deleted;
});

into_dataframe!(
    /// Join to `Vec<Transaction>::into_dataframe()` on `transaction_id`.
    /// `transfer_transaction_id` is a nullable `String` column.
    Subtransaction, |t| {
    "id": id = t.id;
    "transaction_id": transaction_id = t.transaction_id;
    "amount": amount = t.amount;
    "memo": memo = t.memo;
    "payee_id": payee_id = t.payee_id.map(|u| u.to_string());
    "payee_name": payee_name = t.payee_name;
    "category_id": category_id = t.category_id.map(|u| u.to_string());
    "category_name": category_name = t.category_name;
    "transfer_account_id": transfer_account_id = t.transfer_account_id.map(|u| u.to_string());
    "transfer_transaction_id": transfer_transaction_id = t.transfer_transaction_id;
    "deleted": deleted = t.deleted;
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{
        account_fixture, category_fixture, payee_fixture, transaction_fixture,
    };
    use serde_json::json;

    #[test]
    fn account_into_dataframe_has_expected_columns_and_casts() {
        let mut v = account_fixture();
        v["last_reconciled_at"] = json!("2024-03-01T12:00:00Z");
        let account: Account = serde_json::from_value(v).unwrap();
        let df = vec![account].into_dataframe();

        assert_eq!(
            df.get_column_names(),
            vec![
                "id",
                "name",
                "type",
                "on_budget",
                "closed",
                "note",
                "balance",
                "cleared_balance",
                "uncleared_balance",
                "transfer_payee_id",
                "direct_import_linked",
                "direct_import_in_error",
                "last_reconciled_at",
                "debt_original_balance",
                "debt_interest_rates_count",
                "debt_minimum_payments_count",
                "debt_escrow_amounts_count",
                "deleted",
            ]
        );

        let acct_type = df.column("type").unwrap().str().unwrap();
        assert_eq!(acct_type.get(0), Some("Checking"));

        let last_reconciled_at = df.column("last_reconciled_at").unwrap();
        assert_eq!(
            last_reconciled_at.dtype(),
            &DataType::Datetime(TimeUnit::Milliseconds, None)
        );
        let expected_millis = chrono::DateTime::parse_from_rfc3339("2024-03-01T12:00:00Z")
            .unwrap()
            .timestamp_millis();
        let millis = last_reconciled_at
            .cast(&DataType::Int64)
            .unwrap()
            .i64()
            .unwrap()
            .get(0);
        assert_eq!(millis, Some(expected_millis));
    }

    #[test]
    fn account_debt_history_flattens_maps_into_long_format() {
        let mut v = account_fixture();
        v["debt_interest_rates"] = json!({ "2024-01-01": 5000, "2024-02-01": 4500 });
        v["debt_escrow_amounts"] = json!({ "2024-01-01": 1000 });
        let account: Account = serde_json::from_value(v).unwrap();

        let df = account_debt_history(&[account]);

        assert_eq!(df.height(), 3);
        assert_eq!(
            df.get_column_names(),
            vec!["account_id", "kind", "month", "amount"]
        );

        let kind = df.column("kind").unwrap().str().unwrap();
        let interest_rate_count = (0..df.height())
            .filter(|&i| kind.get(i) == Some("interest_rate"))
            .count();
        let escrow_amount_count = (0..df.height())
            .filter(|&i| kind.get(i) == Some("escrow_amount"))
            .count();
        assert_eq!(interest_rate_count, 2);
        assert_eq!(escrow_amount_count, 1);

        let month = df.column("month").unwrap().str().unwrap();
        let amount = df.column("amount").unwrap().i64().unwrap();
        let escrow_row = (0..df.height())
            .find(|&i| kind.get(i) == Some("escrow_amount"))
            .unwrap();
        assert_eq!(month.get(escrow_row), Some("2024-01-01"));
        assert_eq!(amount.get(escrow_row), Some(1000));
    }

    #[test]
    fn account_debt_history_skips_accounts_with_no_debt_maps() {
        let account: Account = serde_json::from_value(account_fixture()).unwrap();
        let df = account_debt_history(&[account]);
        assert_eq!(df.height(), 0);
    }

    #[test]
    fn transaction_into_dataframe_casts_date_correctly() {
        let transaction: Transaction = serde_json::from_value(transaction_fixture()).unwrap();
        let df = vec![transaction].into_dataframe();

        let date = df.column("date").unwrap();
        assert_eq!(date.dtype(), &DataType::Date);
        let days = date.cast(&DataType::Int32).unwrap().i32().unwrap().get(0);
        let expected_days =
            (NaiveDate::from_ymd_opt(2024, 1, 15).unwrap() - epoch()).num_days() as i32;
        assert_eq!(days, Some(expected_days));
    }

    #[test]
    fn category_into_dataframe_handles_nullable_goal_fields() {
        let mut with_goal = category_fixture();
        with_goal["goal_type"] = json!("TB");
        with_goal["goal_target_date"] = json!("2024-06-01");
        let with_goal: Category = serde_json::from_value(with_goal).unwrap();

        let without_goal: Category = serde_json::from_value(category_fixture()).unwrap();

        let df = vec![with_goal, without_goal].into_dataframe();

        let goal_type = df.column("goal_type").unwrap().str().unwrap();
        assert_eq!(goal_type.get(0), Some("TargetBalance"));
        assert_eq!(goal_type.get(1), None);

        let goal_target_date = df.column("goal_target_date").unwrap();
        assert_eq!(goal_target_date.dtype(), &DataType::Date);
        let goal_target_date_i32 = goal_target_date.cast(&DataType::Int32).unwrap();
        let days = goal_target_date_i32.i32().unwrap();
        let expected_days =
            (NaiveDate::from_ymd_opt(2024, 6, 1).unwrap() - epoch()).num_days() as i32;
        assert_eq!(days.get(0), Some(expected_days));
        assert_eq!(days.get(1), None);
    }

    #[test]
    fn payee_into_dataframe_simple_case() {
        let payee: Payee = serde_json::from_value(payee_fixture()).unwrap();
        let df = vec![payee].into_dataframe();

        assert_eq!(
            df.get_column_names(),
            vec!["id", "name", "transfer_account_id", "deleted"]
        );
        let name = df.column("name").unwrap().str().unwrap();
        assert_eq!(name.get(0), Some("Amazon"));
        let deleted = df.column("deleted").unwrap().bool().unwrap();
        assert_eq!(deleted.get(0), Some(false));
    }
}

// TODO: Add an example
