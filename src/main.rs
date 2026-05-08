use rust_ynab::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(std::env::var("YNAB_TOKEN")?)?;

    let plans = client.get_plans().include_accounts().send().await?;
    for plan in plans {
        println!("{}", plan.name);
        for acct in &plan.accounts {
            println!("   {}", acct.name);
        }
    }

    Ok(())
}
