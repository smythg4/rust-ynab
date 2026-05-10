use rust_ynab::Client;
use rust_ynab::PlanId;
use rust_ynab::print_transaction_table;

/// Fetches all transactions for the test plan and prints them as a table.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("YNAB_TOKEN")?;
    let client = Client::new(&token)?;

    let plan_id_str = std::env::var("YNAB_TEST_PLAN_ID")?;
    let plan_id: uuid::Uuid = plan_id_str.parse()?;

    let (txs, _) = client.get_transactions(PlanId::Id(plan_id)).send().await?;

    print_transaction_table(&txs);
    Ok(())
}
