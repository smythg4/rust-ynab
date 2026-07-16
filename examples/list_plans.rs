use rust_ynab::Client;

/// Lists all plans with ID, name, and currency code.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("YNAB_TOKEN")?;
    let client = Client::new(&token)?;
    let plans = client.get_plans().send().await?;

    if plans.is_empty() {
        println!("no plans found");
        return Ok(());
    }

    println!("{:<36}  {:<30}  Currency", "ID", "Name");
    println!(
        "{:<36}  {:<30}  --------",
        "------------------------------------", "------------------------------"
    );
    for plan in plans {
        let name = if plan.name.len() > 30 {
            format!("{}...", &plan.name[..27])
        } else {
            plan.name.clone()
        };
        println!(
            "{:<36}  {:<30}  {}",
            plan.id,
            name,
            plan.currency_format
                .as_ref()
                .map(|cf| cf.iso_code.as_str())
                .unwrap_or("")
        );
    }

    Ok(())
}
