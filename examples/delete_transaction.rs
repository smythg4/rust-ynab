use rust_ynab::Client;
use rust_ynab::PlanId;
use rust_ynab::milliunits_to_amount;

/// Fetches the first transaction from the first account and deletes it.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("YNAB_TOKEN")?;
    let client = Client::new(&token)?;

    let plan_id_str = std::env::var("YNAB_TEST_PLAN_ID")?;
    let (plan, _) = client
        .get_plan(PlanId::Id(plan_id_str.parse()?))
        .send()
        .await?;
    let plan_id = plan.id();

    let (accounts, _) = client.get_accounts(plan_id).send().await?;
    if accounts.is_empty() {
        println!("no accounts found");
        return Ok(());
    }
    let account_id = accounts.first().unwrap().id;

    let (txs, _) = client
        .get_transactions_by_account(plan_id, account_id)
        .send()
        .await?;
    if txs.is_empty() {
        println!("no transactions found");
        return Ok(());
    }

    let tx_id = txs.first().unwrap().id.clone();
    let (tx, _) = client.delete_transaction(plan_id, &tx_id).await?;

    println!("Deleted Transaction\n");
    println!("   {:<10} {}", "ID:", tx.id);
    println!(
        "   {:<10} {}",
        "Account:",
        tx.account_name.as_deref().unwrap_or("")
    );
    println!("   {:<10} {}", "Date:", tx.date);
    println!(
        "   {:<10} ${:.2}",
        "Amount:",
        milliunits_to_amount(tx.amount)
    );
    println!("   {:<10} {}", "Memo:", tx.memo.as_deref().unwrap_or(""));

    Ok(())
}
