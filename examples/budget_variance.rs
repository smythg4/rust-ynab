//! Budget vs. actual variance, by category and month.
//!
//! `Category` amounts (`budgeted`, `activity`, `balance`) are scoped to a single plan month and
//! carry no month of their own. `get_months` (the list endpoint) only returns month summaries —
//! no nested categories — so each month is fetched individually via `get_month`, which does
//! include them. Each month's `categories.into_dataframe()` gets a `month` column stacked onto
//! it, then all months are `concat`-ed into one long table — the same flatten-then-stack shape
//! `account_debt_history` uses internally.
//!
//! Requires the `polars` feature:
//!   cargo run --example budget_variance --features polars
use chrono::{Months, Utc};
use polars::prelude::*;
use rust_ynab::{Client, IntoDataFrame, PlanId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("YNAB_TOKEN")?;
    let client = Client::new(&token)?;

    let plan_id_str = std::env::var("YNAB_TEST_PLAN_ID")?;
    let plan_id = PlanId::Id(plan_id_str.parse()?);

    let cutoff = Utc::now().date_naive() - Months::new(12);

    // `get_months` returns month summaries without categories — only the single-month
    // endpoint includes category detail, so fetch each month individually.
    let (month_summaries, _) = client.get_months(plan_id).send().await?;

    let mut per_month = Vec::new();
    for summary in month_summaries.iter().filter(|s| s.month >= cutoff) {
        let m = client.get_month(plan_id, summary.month).await?;
        per_month.push(
            m.categories
                .into_dataframe()
                .lazy()
                .with_column(lit(m.month).cast(DataType::Date).alias("month")),
        );
    }

    let result = concat(&per_month, UnionArgs::default())?
        .filter(col("hidden").eq(lit(false)))
        .filter(col("deleted").eq(lit(false)))
        .with_column(((col("budgeted") + col("activity")) / lit(1000.0)).alias("variance"))
        .select([
            col("month"),
            col("name").alias("category"),
            (col("budgeted") / lit(1000.0)).alias("budgeted"),
            (col("activity") / lit(-1000.0)).alias("spent"),
            col("variance"),
        ])
        .sort(["variance"], SortMultipleOptions::default())
        .collect()?;

    println!("{result}");

    Ok(())
}
