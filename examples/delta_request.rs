use rust_ynab::Client;
use rust_ynab::NewTransaction;
use rust_ynab::PlanId;
use rust_ynab::milliunits_to_amount;

/// Fetches all transactions, creates one, then fetches again using server knowledge
/// to demonstrate that only the newly created transaction is returned in the delta.
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

    let (txs, sk) = client.get_transactions(plan_id).send().await?;
    println!(
        "Initial fetch (server knowledge: {})  —  {} transactions\n",
        sk,
        txs.len()
    );

    let (accounts, _) = client.get_accounts(plan_id).send().await?;
    if accounts.is_empty() {
        println!("no accounts found");
        return Ok(());
    }
    let account_id = accounts.first().unwrap().id;

    let new_tx = NewTransaction {
        account_id,
        date: chrono::Local::now().date_naive(),
        amount: Some(1000),
        payee_id: None,
        payee_name: None,
        category_id: None,
        memo: Some("delta request test".to_string()),
        cleared: Some(rust_ynab::ClearedStatus::Uncleared),
        approved: Some(false),
        flag_color: None,
        import_id: None,
        subtransactions: None,
    };

    let resp = client.create_transaction(plan_id, new_tx).await?;
    let created = resp.transaction.ok_or("no transaction in response")?;
    println!("Created transaction: {}\n", created.id);

    let (delta_txs, new_sk) = client
        .get_transactions(plan_id)
        .with_server_knowledge(sk)
        .send()
        .await?;

    println!("Delta fetch (since knowledge: {})\n", sk);
    if delta_txs.is_empty() {
        println!("no changes since server knowledge {}", sk);
    } else {
        println!("{} new/changed transactions:\n", delta_txs.len());
        for tx in &delta_txs {
            println!(
                "   {:<36}  {:<12}  {:>10}  {}",
                tx.id,
                tx.date,
                format!("${:.2}", milliunits_to_amount(tx.amount)),
                tx.payee_name.as_deref().unwrap_or(""),
            );
        }
    }
    println!("\nNew server knowledge: {}", new_sk);

    Ok(())
}
