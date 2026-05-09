use rust_ynab::Client;
use rust_ynab::NewTransaction;
use rust_ynab::PlanId;
use rust_ynab::milliunits_to_amount;

/// Creates a single transaction and prints the result.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("YNAB_TOKEN")?;
    let client = Client::new(&token)?;

    let plan_id_str = std::env::var("YNAB_TEST_PLAN_ID")?;
    let (plan, _) = client
        .get_plan(PlanId::Id(plan_id_str.parse()?))
        .send()
        .await?;
    let (accounts, _) = client.get_accounts(plan.id()).send().await?;

    if accounts.is_empty() {
        println!("no accounts found");
        return Ok(());
    }

    let account_id = accounts.last().unwrap().id;

    let tx = NewTransaction {
        account_id,
        date: chrono::Local::now().date_naive(),
        amount: Some(1000),
        payee_id: None,
        payee_name: None,
        category_id: None,
        memo: Some("test transaction".to_string()),
        cleared: Some(rust_ynab::ClearedStatus::Uncleared),
        approved: Some(false),
        flag_color: None,
        import_id: None,
        subtransactions: None,
    };

    let resp = client.create_transaction(plan.id(), tx).await?;
    let tx = resp.transaction.ok_or("no transaction in response")?;

    println!("Created Transaction\n");
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
