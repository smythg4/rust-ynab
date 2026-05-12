use rust_ynab::Client;
use rust_ynab::PlanId;
use rust_ynab::SaveTransactionWithIdOrImportId;
use rust_ynab::milliunits_to_amount;
use std::collections::HashMap;

/// Fetches the two most recent transactions, appends " (updated)" to their memos
/// via a PATCH request, and prints the before and after state.
///
/// Only the memo field is sent; all other fields are left unchanged by the API.
///
/// Note: running this example multiple times will append " (updated)" repeatedly.
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
    if txs.len() < 2 {
        println!("not enough transactions found");
        return Ok(());
    }

    let originals = &txs[txs.len() - 2..];
    let mut old_memos: HashMap<String, String> = HashMap::new();
    let mut patches = Vec::new();

    for tx in originals {
        let old_memo = tx.memo.as_deref().unwrap_or("").to_string();
        let new_memo = format!("{} (updated)", old_memo);
        old_memos.insert(tx.id.clone(), old_memo);
        patches.push(SaveTransactionWithIdOrImportId {
            id: Some(tx.id.parse()?),
            memo: Some(new_memo),
            import_id: None,
            account_id: None,
            date: None,
            amount: None,
            payee_id: None,
            payee_name: None,
            category_id: None,
            cleared: None,
            approved: None,
            flag_color: None,
            subtransactions: None,
        });
    }

    let resp = client.update_transactions(plan_id, patches).await?;
    let updated = resp.transactions;

    println!("Updated Transactions\n");
    println!("Plan: {}\n", plan.plan.name);
    for tx in &updated {
        println!("   {:<10} {}", "ID:", tx.id);
        println!("   {:<10} {}", "Account:", tx.account_name);
        println!("   {:<10} {}", "Date:", tx.date);
        println!(
            "   {:<10} ${:.2}",
            "Amount:",
            milliunits_to_amount(tx.amount)
        );
        println!(
            "   {:<10} {}  ->  {}",
            "Memo:",
            old_memos
                .get(tx.id.as_str())
                .map(|s| s.as_str())
                .unwrap_or(""),
            tx.memo.as_deref().unwrap_or(""),
        );
        println!();
    }

    Ok(())
}
