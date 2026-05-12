use rust_ynab::Client;
use rust_ynab::PlanId;
use rust_ynab::Transaction;
use rust_ynab::milliunits_to_amount;

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max
        && let Some(trunc) = s.char_indices().nth(max - 3).map(|(i, _)| &s[..i])
    {
        format!("{trunc}...")
    } else {
        s.to_string()
    }
}

fn print_transaction_table(transactions: &[Transaction]) {
    println!(
        "{:<12}  {:<20}  {:<25}  {:>13}",
        "Date", "Account", "Payee", "Amount"
    );
    println!(
        "{:<12}  {:<20}  {:<25}  {:>13}",
        "------------", "--------------------", "-------------------------", "-------------"
    );
    for tx in transactions {
        let mut account = tx.account_name.clone();
        account.truncate(20);
        let payee = tx
            .payee_name
            .as_deref()
            .map(|s| truncate(s, 25))
            .unwrap_or_default();
        println!(
            "{:<12}  {:<20}  {:<25}  {:>13}",
            tx.date,
            account,
            payee,
            format!("${:.2}", milliunits_to_amount(tx.amount))
        );
    }
}

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
