//! Debt account history — interest rate, minimum payment, and escrow amount over time.
//!
//! `Account`'s three debt maps are date-keyed and variable-length per account, so they don't fit
//! a flat column on `Account::into_dataframe()` (only their entry counts appear there).
//! `account_debt_history` flattens them into a long-format table (`account_id`, `kind`, `month`,
//! `amount`) instead. This joins that table against `Account::into_dataframe()` to bring in
//! account names, and filters to the trailing 12 months.
//!
//! Requires the `polars` feature:
//!   cargo run --example debt_account_history --features polars
use chrono::{Months, Utc};
use polars::prelude::*;
use rust_ynab::{Client, IntoDataFrame, PlanId, account_debt_history};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("YNAB_TOKEN")?;
    let client = Client::new(&token)?;

    let plan_id_str = std::env::var("YNAB_TEST_PLAN_ID")?;
    let plan_id = PlanId::Id(plan_id_str.parse()?);

    let (accounts, _) = client.get_accounts(plan_id).send().await?;

    let cutoff = Utc::now().date_naive() - Months::new(12);

    let names = accounts
        .clone()
        .into_dataframe()
        .lazy()
        .select([col("id").alias("account_id"), col("name")]);

    let result = account_debt_history(&accounts)
        .lazy()
        .with_column(
            col("month")
                .str()
                .to_date(StrptimeOptions::default())
                .alias("month"),
        )
        .filter(col("month").gt_eq(lit(cutoff)))
        .left_join(names, col("account_id"), col("account_id"))
        .with_column((col("amount") / lit(1000.0)).alias("value"))
        .select([col("name"), col("kind"), col("month"), col("value")])
        .sort(["name", "kind", "month"], SortMultipleOptions::default())
        .collect()?;

    println!("{result}");

    Ok(())
}
