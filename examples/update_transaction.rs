use rust_ynab::Client;
use rust_ynab::ExistingTransaction;
use rust_ynab::PlanId;
use rust_ynab::milliunits_to_amount;

/// Fetches the most recent transaction from the test plan, appends " (updated)"
/// to its memo, and prints the before and after state.
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

    let (txs, _) = client.get_transactions(plan_id).send().await?;
    if txs.is_empty() {
        println!("no transactions found");
        return Ok(());
    }
    let tx = txs.last().unwrap();

    let old_memo = tx.memo.as_deref().unwrap_or("").to_string();
    let new_memo = format!("{} (updated)", old_memo);

    let updated_tx = ExistingTransaction {
        account_id: None,
        date: None,
        amount: None,
        payee_id: None,
        payee_name: None,
        category_id: None,
        memo: Some(new_memo),
        cleared: None,
        approved: None,
        flag_color: None,
        subtransactions: None,
    };

    let (tx_resp, _) = client
        .update_transaction(plan_id, &tx.id, updated_tx)
        .await?;

    println!("Updated Transaction\n");
    println!("   {:<10} {}", "ID:", tx_resp.id);
    println!("   {:<10} {}", "Account:", tx_resp.account_name);
    println!("   {:<10} {}", "Date:", tx_resp.date);
    println!(
        "   {:<10} ${:.2}",
        "Amount:",
        milliunits_to_amount(tx_resp.amount)
    );
    println!(
        "   {:<10} {}  ->  {}",
        "Memo:",
        old_memo,
        tx_resp.memo.as_deref().unwrap_or("")
    );

    Ok(())
}
