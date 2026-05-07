use rust_ynab::ynab::client::Client;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(std::env::var("YNAB_TOKEN")?)?;

    let (plan, sk) = client.get_last_plan_used(&[]).await?;

    let months = client.get_months(plan.plan.id, Some(sk)).await?;

    for month in months {
        println!("{}: {:?}", month.month, month.note)
    }
    Ok(())
}
