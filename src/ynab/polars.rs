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
    Account, Category, CategoryGroup, MoneyMovement, MoneyMovementGroup, Month, Payee,
    PayeeLocation, Plan, ScheduledSubtransaction, ScheduledTransaction, Subtransaction,
    Transaction,
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

/// `type` is stringified. `last_reconciled_at` is a `Datetime(Milliseconds)` column.
impl IntoDataFrame for Vec<Account> {
    fn into_dataframe(self) -> DataFrame {
        let accounts = self;
        let size = accounts.len();
        let mut id = Vec::with_capacity(size);
        let mut name = Vec::with_capacity(size);
        let mut acct_type = Vec::with_capacity(size);
        let mut on_budget = Vec::with_capacity(size);
        let mut closed = Vec::with_capacity(size);
        let mut note = Vec::with_capacity(size);
        let mut balance = Vec::with_capacity(size);
        let mut cleared_balance = Vec::with_capacity(size);
        let mut uncleared_balance = Vec::with_capacity(size);
        let mut transfer_payee_id = Vec::with_capacity(size);
        let mut direct_import_linked = Vec::with_capacity(size);
        let mut direct_import_in_error = Vec::with_capacity(size);
        let mut last_reconciled_at = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        for a in accounts {
            id.push(a.id.to_string());
            name.push(a.name);
            acct_type.push(format!("{:?}", a.acct_type));
            on_budget.push(a.on_budget);
            closed.push(a.closed);
            note.push(a.note);
            balance.push(a.balance);
            cleared_balance.push(a.cleared_balance);
            uncleared_balance.push(a.uncleared_balance);
            transfer_payee_id.push(a.transfer_payee_id.map(|tpi| tpi.to_string()));
            direct_import_linked.push(a.direct_import_linked);
            direct_import_in_error.push(a.direct_import_in_error);
            last_reconciled_at.push(a.last_reconciled_at.map(|lra| lra.timestamp_millis()));
            deleted.push(a.deleted);
        }
        let id = Series::new("id".into(), id);
        let name = Series::new("name".into(), name);
        let acct_type = Series::new("type".into(), acct_type);
        let on_budget = Series::new("on_budget".into(), on_budget);
        let closed = Series::new("closed".into(), closed);
        let note = Series::new("note".into(), note);
        let balance = Series::new("balance".into(), balance);
        let cleared_balance = Series::new("cleared_balance".into(), cleared_balance);
        let uncleared_balance = Series::new("uncleared_balance".into(), uncleared_balance);
        let transfer_payee_id = Series::new("transfer_payee_id".into(), transfer_payee_id);
        let direct_import_linked = Series::new("direct_import_linked".into(), direct_import_linked);
        let direct_import_in_error =
            Series::new("direct_import_in_error".into(), direct_import_in_error);
        let last_reconciled_at = Series::new("last_reconciled_at".into(), last_reconciled_at)
            .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
            .unwrap();
        let deleted = Series::new("deleted".into(), deleted);

        DataFrame::new(
            size,
            vec![
                id.into(),
                name.into(),
                acct_type.into(),
                on_budget.into(),
                closed.into(),
                note.into(),
                balance.into(),
                cleared_balance.into(),
                uncleared_balance.into(),
                transfer_payee_id.into(),
                direct_import_linked.into(),
                direct_import_in_error.into(),
                last_reconciled_at.into(),
                deleted.into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// `date` is a `Date` column. `cleared` and `flag_color` are stringified.
/// `subtransactions` is dropped — use `Vec<Subtransaction>::into_dataframe()` and join on `transaction_id`.
impl IntoDataFrame for Vec<Transaction> {
    fn into_dataframe(self) -> DataFrame {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut account_name = Vec::with_capacity(size);
        let mut date = Vec::with_capacity(size);
        let mut amount = Vec::with_capacity(size);
        let mut memo = Vec::with_capacity(size);
        let mut cleared = Vec::with_capacity(size);
        let mut approved = Vec::with_capacity(size);
        let mut flag_color = Vec::with_capacity(size);
        let mut flag_name = Vec::with_capacity(size);
        let mut account_id = Vec::with_capacity(size);
        let mut payee_id = Vec::with_capacity(size);
        let mut payee_name = Vec::with_capacity(size);
        let mut category_id = Vec::with_capacity(size);
        let mut category_name = Vec::with_capacity(size);
        let mut matched_transaction_id = Vec::with_capacity(size);
        let mut import_id = Vec::with_capacity(size);
        let mut import_payee_name = Vec::with_capacity(size);
        let mut import_payee_name_original = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        for t in self {
            id.push(t.id);
            account_name.push(t.account_name);
            date.push((t.date - epoch).num_days() as i32);
            amount.push(t.amount);
            memo.push(t.memo);
            cleared.push(format!("{:?}", t.cleared));
            approved.push(t.approved);
            flag_color.push(t.flag_color.map(|f| format!("{:?}", f)));
            flag_name.push(t.flag_name);
            account_id.push(t.account_id.to_string());
            payee_id.push(t.payee_id.map(|u| u.to_string()));
            payee_name.push(t.payee_name);
            category_id.push(t.category_id.map(|u| u.to_string()));
            category_name.push(t.category_name);
            matched_transaction_id.push(t.matched_transaction_id);
            import_id.push(t.import_id);
            import_payee_name.push(t.import_payee_name);
            import_payee_name_original.push(t.import_payee_name_original);
            deleted.push(t.deleted);
        }
        let date = Series::new("date".into(), date)
            .cast(&DataType::Date)
            .unwrap();
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                Series::new("account_name".into(), account_name).into(),
                date.into(),
                Series::new("amount".into(), amount).into(),
                Series::new("memo".into(), memo).into(),
                Series::new("cleared".into(), cleared).into(),
                Series::new("approved".into(), approved).into(),
                Series::new("flag_color".into(), flag_color).into(),
                Series::new("flag_name".into(), flag_name).into(),
                Series::new("account_id".into(), account_id).into(),
                Series::new("payee_id".into(), payee_id).into(),
                Series::new("payee_name".into(), payee_name).into(),
                Series::new("category_id".into(), category_id).into(),
                Series::new("category_name".into(), category_name).into(),
                Series::new("matched_transaction_id".into(), matched_transaction_id).into(),
                Series::new("import_id".into(), import_id).into(),
                Series::new("import_payee_name".into(), import_payee_name).into(),
                Series::new(
                    "import_payee_name_original".into(),
                    import_payee_name_original,
                )
                .into(),
                Series::new("deleted".into(), deleted).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// `categories` is dropped — use `Vec<Category>::into_dataframe()` and join on `category_group_id`.
/// A `category_count` column is included as a convenience.
impl IntoDataFrame for Vec<CategoryGroup> {
    fn into_dataframe(self) -> DataFrame {
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut name = Vec::with_capacity(size);
        let mut hidden = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        let mut category_count = Vec::with_capacity(size);
        for g in self {
            id.push(g.id.to_string());
            name.push(g.name);
            hidden.push(g.hidden);
            deleted.push(g.deleted);
            category_count.push(g.categories.len() as u32);
        }
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                Series::new("name".into(), name).into(),
                Series::new("hidden".into(), hidden).into(),
                Series::new("deleted".into(), deleted).into(),
                Series::new("category_count".into(), category_count).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// All goal date fields are `Date` columns. `goal_snoozed_at` is `Datetime(Milliseconds)`.
/// `goal_type` is stringified. `usize` goal fields are cast to `i32`.
impl IntoDataFrame for Vec<Category> {
    fn into_dataframe(self) -> DataFrame {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut category_group_id = Vec::with_capacity(size);
        let mut category_group_name = Vec::with_capacity(size);
        let mut name = Vec::with_capacity(size);
        let mut hidden = Vec::with_capacity(size);
        let mut original_category_group_id = Vec::with_capacity(size);
        let mut note = Vec::with_capacity(size);
        let mut budgeted = Vec::with_capacity(size);
        let mut activity = Vec::with_capacity(size);
        let mut balance = Vec::with_capacity(size);
        let mut goal_type = Vec::with_capacity(size);
        let mut goal_needs_whole_amount = Vec::with_capacity(size);
        let mut goal_day = Vec::with_capacity(size);
        let mut goal_cadence = Vec::with_capacity(size);
        let mut goal_cadence_frequency = Vec::with_capacity(size);
        let mut goal_creation_month = Vec::with_capacity(size);
        let mut goal_target = Vec::with_capacity(size);
        let mut goal_target_date = Vec::with_capacity(size);
        let mut goal_target_month = Vec::with_capacity(size);
        let mut goal_percentage_complete = Vec::with_capacity(size);
        let mut goal_months_to_budget = Vec::with_capacity(size);
        let mut goal_under_funded = Vec::with_capacity(size);
        let mut goal_overall_funded = Vec::with_capacity(size);
        let mut goal_overall_left = Vec::with_capacity(size);
        let mut goal_snoozed_at = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        for c in self {
            id.push(c.id.to_string());
            category_group_id.push(c.category_group_id.to_string());
            category_group_name.push(c.category_group_name);
            name.push(c.name);
            hidden.push(c.hidden);
            original_category_group_id.push(c.original_category_group_id.map(|u| u.to_string()));
            note.push(c.note);
            budgeted.push(c.budgeted);
            activity.push(c.activity);
            balance.push(c.balance);
            goal_type.push(c.goal_type.map(|g| format!("{:?}", g)));
            goal_needs_whole_amount.push(c.goal_needs_whole_amount);
            goal_day.push(c.goal_day.map(|v| v as i32));
            goal_cadence.push(c.goal_cadence.map(|v| v as i32));
            goal_cadence_frequency.push(c.goal_cadence_frequency.map(|v| v as i32));
            goal_creation_month.push(c.goal_creation_month.map(|d| (d - epoch).num_days() as i32));
            goal_target.push(c.goal_target);
            goal_target_date.push(c.goal_target_date.map(|d| (d - epoch).num_days() as i32));
            goal_target_month.push(c.goal_target_month.map(|d| (d - epoch).num_days() as i32));
            goal_percentage_complete.push(c.goal_percentage_complete.map(|v| v as i32));
            goal_months_to_budget.push(c.goal_months_to_budget.map(|v| v as i32));
            goal_under_funded.push(c.goal_under_funded);
            goal_overall_funded.push(c.goal_overall_funded);
            goal_overall_left.push(c.goal_overall_left);
            goal_snoozed_at.push(c.goal_snoozed_at.map(|dt| dt.timestamp_millis()));
            deleted.push(c.deleted);
        }
        let goal_creation_month = Series::new("goal_creation_month".into(), goal_creation_month)
            .cast(&DataType::Date)
            .unwrap();
        let goal_target_date = Series::new("goal_target_date".into(), goal_target_date)
            .cast(&DataType::Date)
            .unwrap();
        let goal_target_month = Series::new("goal_target_month".into(), goal_target_month)
            .cast(&DataType::Date)
            .unwrap();
        let goal_snoozed_at = Series::new("goal_snoozed_at".into(), goal_snoozed_at)
            .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
            .unwrap();
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                Series::new("category_group_id".into(), category_group_id).into(),
                Series::new("category_group_name".into(), category_group_name).into(),
                Series::new("name".into(), name).into(),
                Series::new("hidden".into(), hidden).into(),
                Series::new(
                    "original_category_group_id".into(),
                    original_category_group_id,
                )
                .into(),
                Series::new("note".into(), note).into(),
                Series::new("budgeted".into(), budgeted).into(),
                Series::new("activity".into(), activity).into(),
                Series::new("balance".into(), balance).into(),
                Series::new("goal_type".into(), goal_type).into(),
                Series::new("goal_needs_whole_amount".into(), goal_needs_whole_amount).into(),
                Series::new("goal_day".into(), goal_day).into(),
                Series::new("goal_cadence".into(), goal_cadence).into(),
                Series::new("goal_cadence_frequency".into(), goal_cadence_frequency).into(),
                goal_creation_month.into(),
                Series::new("goal_target".into(), goal_target).into(),
                goal_target_date.into(),
                goal_target_month.into(),
                Series::new("goal_percentage_complete".into(), goal_percentage_complete).into(),
                Series::new("goal_months_to_budget".into(), goal_months_to_budget).into(),
                Series::new("goal_under_funded".into(), goal_under_funded).into(),
                Series::new("goal_overall_funded".into(), goal_overall_funded).into(),
                Series::new("goal_overall_left".into(), goal_overall_left).into(),
                goal_snoozed_at.into(),
                Series::new("deleted".into(), deleted).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// `month` is a `Date` column. `categories` is dropped — use `Vec<Category>::into_dataframe()`.
/// A `category_count` column is included as a convenience.
impl IntoDataFrame for Vec<Month> {
    fn into_dataframe(self) -> DataFrame {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let size = self.len();
        let mut month = Vec::with_capacity(size);
        let mut note = Vec::with_capacity(size);
        let mut income = Vec::with_capacity(size);
        let mut budgeted = Vec::with_capacity(size);
        let mut activity = Vec::with_capacity(size);
        let mut to_be_budgeted = Vec::with_capacity(size);
        let mut age_of_money = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        let mut category_count = Vec::with_capacity(size);
        for m in self {
            month.push((m.month - epoch).num_days() as i32);
            note.push(m.note);
            income.push(m.income);
            budgeted.push(m.budgeted);
            activity.push(m.activity);
            to_be_budgeted.push(m.to_be_budgeted);
            age_of_money.push(m.age_of_money.map(|v| v as i32));
            deleted.push(m.deleted);
            category_count.push(m.categories.len() as u32);
        }
        let month = Series::new("month".into(), month)
            .cast(&DataType::Date)
            .unwrap();
        DataFrame::new(
            size,
            vec![
                month.into(),
                Series::new("note".into(), note).into(),
                Series::new("income".into(), income).into(),
                Series::new("budgeted".into(), budgeted).into(),
                Series::new("activity".into(), activity).into(),
                Series::new("to_be_budgeted".into(), to_be_budgeted).into(),
                Series::new("age_of_money".into(), age_of_money).into(),
                Series::new("deleted".into(), deleted).into(),
                Series::new("category_count".into(), category_count).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// `month` is a nullable `Date` column. `moved_at` is a nullable `Datetime(Milliseconds)` column.
impl IntoDataFrame for Vec<MoneyMovement> {
    fn into_dataframe(self) -> DataFrame {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut month = Vec::with_capacity(size);
        let mut moved_at = Vec::with_capacity(size);
        let mut note = Vec::with_capacity(size);
        let mut money_movement_group_id = Vec::with_capacity(size);
        let mut performed_by_user_id = Vec::with_capacity(size);
        let mut from_category_id = Vec::with_capacity(size);
        let mut to_category_id = Vec::with_capacity(size);
        let mut amount = Vec::with_capacity(size);
        for m in self {
            id.push(m.id.to_string());
            month.push(m.month.map(|d| (d - epoch).num_days() as i32));
            moved_at.push(m.moved_at.map(|dt| dt.timestamp_millis()));
            note.push(m.note);
            money_movement_group_id.push(m.money_movement_group_id.map(|u| u.to_string()));
            performed_by_user_id.push(m.performed_by_user_id.map(|u| u.to_string()));
            from_category_id.push(m.from_category_id.map(|u| u.to_string()));
            to_category_id.push(m.to_category_id.map(|u| u.to_string()));
            amount.push(m.amount);
        }
        let month = Series::new("month".into(), month)
            .cast(&DataType::Date)
            .unwrap();
        let moved_at = Series::new("moved_at".into(), moved_at)
            .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
            .unwrap();
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                month.into(),
                moved_at.into(),
                Series::new("note".into(), note).into(),
                Series::new("money_movement_group_id".into(), money_movement_group_id).into(),
                Series::new("performed_by_user_id".into(), performed_by_user_id).into(),
                Series::new("from_category_id".into(), from_category_id).into(),
                Series::new("to_category_id".into(), to_category_id).into(),
                Series::new("amount".into(), amount).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// `group_created_at` is a `Datetime(Milliseconds)` column. `month` is a `Date` column.
impl IntoDataFrame for Vec<MoneyMovementGroup> {
    fn into_dataframe(self) -> DataFrame {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut group_created_at = Vec::with_capacity(size);
        let mut month = Vec::with_capacity(size);
        let mut note = Vec::with_capacity(size);
        let mut performed_by_user_id = Vec::with_capacity(size);
        for g in self {
            id.push(g.id.to_string());
            group_created_at.push(g.group_created_at.timestamp_millis());
            month.push((g.month - epoch).num_days() as i32);
            note.push(g.note);
            performed_by_user_id.push(g.performed_by_user_id.map(|u| u.to_string()));
        }
        let group_created_at = Series::new("group_created_at".into(), group_created_at)
            .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
            .unwrap();
        let month = Series::new("month".into(), month)
            .cast(&DataType::Date)
            .unwrap();
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                group_created_at.into(),
                month.into(),
                Series::new("note".into(), note).into(),
                Series::new("performed_by_user_id".into(), performed_by_user_id).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// `transfer_account_id` is a nullable `String` column.
impl IntoDataFrame for Vec<Payee> {
    fn into_dataframe(self) -> DataFrame {
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut name = Vec::with_capacity(size);
        let mut transfer_account_id = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        for p in self {
            id.push(p.id.to_string());
            name.push(p.name);
            transfer_account_id.push(p.transfer_account_id.map(|u| u.to_string()));
            deleted.push(p.deleted);
        }
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                Series::new("name".into(), name).into(),
                Series::new("transfer_account_id".into(), transfer_account_id).into(),
                Series::new("deleted".into(), deleted).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// `latitude` and `longitude` are `String` columns as returned by the API.
impl IntoDataFrame for Vec<PayeeLocation> {
    fn into_dataframe(self) -> DataFrame {
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut payee_id = Vec::with_capacity(size);
        let mut latitude = Vec::with_capacity(size);
        let mut longitude = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        for p in self {
            id.push(p.id.to_string());
            payee_id.push(p.payee_id.to_string());
            latitude.push(p.latitude);
            longitude.push(p.longitude);
            deleted.push(p.deleted);
        }
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                Series::new("payee_id".into(), payee_id).into(),
                Series::new("latitude".into(), latitude).into(),
                Series::new("longitude".into(), longitude).into(),
                Series::new("deleted".into(), deleted).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

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

/// `date_first` and `date_next` are `Date` columns. `frequency` and `flag_color` are stringified.
/// `subtransactions` is dropped — use `Vec<ScheduledSubtransaction>::into_dataframe()` and join on `scheduled_transaction_id`.
/// A `subtransaction_count` column is included as a convenience.
impl IntoDataFrame for Vec<ScheduledTransaction> {
    fn into_dataframe(self) -> DataFrame {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut date_first = Vec::with_capacity(size);
        let mut date_next = Vec::with_capacity(size);
        let mut frequency = Vec::with_capacity(size);
        let mut amount = Vec::with_capacity(size);
        let mut memo = Vec::with_capacity(size);
        let mut flag_color = Vec::with_capacity(size);
        let mut flag_name = Vec::with_capacity(size);
        let mut account_id = Vec::with_capacity(size);
        let mut account_name = Vec::with_capacity(size);
        let mut payee_id = Vec::with_capacity(size);
        let mut payee_name = Vec::with_capacity(size);
        let mut category_id = Vec::with_capacity(size);
        let mut category_name = Vec::with_capacity(size);
        let mut transfer_account_id = Vec::with_capacity(size);
        let mut subtransaction_count = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        for t in self {
            id.push(t.id.to_string());
            date_first.push((t.date_first - epoch).num_days() as i32);
            date_next.push((t.date_next - epoch).num_days() as i32);
            frequency.push(format!("{:?}", t.frequency));
            amount.push(t.amount);
            memo.push(t.memo);
            flag_color.push(t.flag_color.map(|f| format!("{:?}", f)));
            flag_name.push(t.flag_name);
            account_id.push(t.account_id.to_string());
            account_name.push(t.account_name);
            payee_id.push(t.payee_id.map(|u| u.to_string()));
            payee_name.push(t.payee_name);
            category_id.push(t.category_id.map(|u| u.to_string()));
            category_name.push(t.category_name);
            transfer_account_id.push(t.transfer_account_id.map(|u| u.to_string()));
            subtransaction_count.push(t.subtransactions.len() as u32);
            deleted.push(t.deleted);
        }
        let date_first = Series::new("date_first".into(), date_first)
            .cast(&DataType::Date)
            .unwrap();
        let date_next = Series::new("date_next".into(), date_next)
            .cast(&DataType::Date)
            .unwrap();
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                date_first.into(),
                date_next.into(),
                Series::new("frequency".into(), frequency).into(),
                Series::new("amount".into(), amount).into(),
                Series::new("memo".into(), memo).into(),
                Series::new("flag_color".into(), flag_color).into(),
                Series::new("flag_name".into(), flag_name).into(),
                Series::new("account_id".into(), account_id).into(),
                Series::new("account_name".into(), account_name).into(),
                Series::new("payee_id".into(), payee_id).into(),
                Series::new("payee_name".into(), payee_name).into(),
                Series::new("category_id".into(), category_id).into(),
                Series::new("category_name".into(), category_name).into(),
                Series::new("transfer_account_id".into(), transfer_account_id).into(),
                Series::new("subtransaction_count".into(), subtransaction_count).into(),
                Series::new("deleted".into(), deleted).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// Join to `Vec<ScheduledTransaction>::into_dataframe()` on `scheduled_transaction_id`.
impl IntoDataFrame for Vec<ScheduledSubtransaction> {
    fn into_dataframe(self) -> DataFrame {
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut scheduled_transaction_id = Vec::with_capacity(size);
        let mut amount = Vec::with_capacity(size);
        let mut memo = Vec::with_capacity(size);
        let mut payee_id = Vec::with_capacity(size);
        let mut payee_name = Vec::with_capacity(size);
        let mut category_id = Vec::with_capacity(size);
        let mut category_name = Vec::with_capacity(size);
        let mut transfer_account_id = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        for t in self {
            id.push(t.id.to_string());
            scheduled_transaction_id.push(t.scheduled_transaction_id.to_string());
            amount.push(t.amount);
            memo.push(t.memo);
            payee_id.push(t.payee_id.map(|u| u.to_string()));
            payee_name.push(t.payee_name);
            category_id.push(t.category_id.map(|u| u.to_string()));
            category_name.push(t.category_name);
            transfer_account_id.push(t.transfer_account_id.map(|u| u.to_string()));
            deleted.push(t.deleted);
        }
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                Series::new("scheduled_transaction_id".into(), scheduled_transaction_id).into(),
                Series::new("amount".into(), amount).into(),
                Series::new("memo".into(), memo).into(),
                Series::new("payee_id".into(), payee_id).into(),
                Series::new("payee_name".into(), payee_name).into(),
                Series::new("category_id".into(), category_id).into(),
                Series::new("category_name".into(), category_name).into(),
                Series::new("transfer_account_id".into(), transfer_account_id).into(),
                Series::new("deleted".into(), deleted).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

/// Join to `Vec<Transaction>::into_dataframe()` on `transaction_id`.
/// `transfer_transaction_id` is a nullable `String` column.
impl IntoDataFrame for Vec<Subtransaction> {
    fn into_dataframe(self) -> DataFrame {
        let size = self.len();
        let mut id = Vec::with_capacity(size);
        let mut transaction_id = Vec::with_capacity(size);
        let mut amount = Vec::with_capacity(size);
        let mut memo = Vec::with_capacity(size);
        let mut payee_id = Vec::with_capacity(size);
        let mut payee_name = Vec::with_capacity(size);
        let mut category_id = Vec::with_capacity(size);
        let mut category_name = Vec::with_capacity(size);
        let mut transfer_account_id = Vec::with_capacity(size);
        let mut transfer_transaction_id = Vec::with_capacity(size);
        let mut deleted = Vec::with_capacity(size);
        for t in self {
            id.push(t.id);
            transaction_id.push(t.transaction_id);
            amount.push(t.amount);
            memo.push(t.memo);
            payee_id.push(t.payee_id.map(|u| u.to_string()));
            payee_name.push(t.payee_name);
            category_id.push(t.category_id.map(|u| u.to_string()));
            category_name.push(t.category_name);
            transfer_account_id.push(t.transfer_account_id.map(|u| u.to_string()));
            transfer_transaction_id.push(t.transfer_transaction_id);
            deleted.push(t.deleted);
        }
        DataFrame::new(
            size,
            vec![
                Series::new("id".into(), id).into(),
                Series::new("transaction_id".into(), transaction_id).into(),
                Series::new("amount".into(), amount).into(),
                Series::new("memo".into(), memo).into(),
                Series::new("payee_id".into(), payee_id).into(),
                Series::new("payee_name".into(), payee_name).into(),
                Series::new("category_id".into(), category_id).into(),
                Series::new("category_name".into(), category_name).into(),
                Series::new("transfer_account_id".into(), transfer_account_id).into(),
                Series::new("transfer_transaction_id".into(), transfer_transaction_id).into(),
                Series::new("deleted".into(), deleted).into(),
            ],
        )
        .expect("all columns must have equal length")
    }
}

// TODO: Add tests and examples
