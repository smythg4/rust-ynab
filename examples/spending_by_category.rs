//! Spending by category, by month — the "where does my money go" analysis.
//!
//! Requires the `polars` feature:
//!   cargo run --example spending_by_category --features polars
use chrono::{Months, Utc};
use polars::prelude::*;
use rust_ynab::{Client, IntoDataFrame, PlanId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("YNAB_TOKEN")?;
    let client = Client::new(&token)?;

    let plan_id_str = std::env::var("YNAB_TEST_PLAN_ID")?;
    let plan_id = PlanId::Id(plan_id_str.parse()?);

    let (transactions, _) = client.get_transactions(plan_id).send().await?;

    let cutoff = Utc::now().date_naive() - Months::new(12);

    let result = transactions
        .into_dataframe()
        .lazy()
        // ignore deleted transactions
        .filter(col("deleted").eq(lit(false)))
        // outflows are negative milliunits; keep only spending, not income/transfers in
        .filter(col("amount").lt(lit(0i64)))
        // only grab transactions for the past 12 months
        .filter(col("date").gt_eq(lit(cutoff)))
        // remove any transactions from the "Inflow" categories
        .filter(col("category_name").str().starts_with(lit("Inflow")).not())
        // transform `date` into month
        .with_column(col("date").dt().month().alias("month"))
        .group_by([col("month"), col("category_name")])
        .agg([(col("amount").sum() / lit(-1000.0)).alias("total_spent")])
        .sort(
            ["total_spent", "month"],
            SortMultipleOptions::default().with_order_descending_multi([true, false]),
        )
        .collect()?;

    println!("{result}");

    Ok(())
}
