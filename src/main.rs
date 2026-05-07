use rust_ynab::PlanId;
use rust_ynab::ynab::client::Client;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(std::env::var("YNAB_TOKEN")?)?;

    let (plan, sk) = client.get_plan(PlanId::LastUsed).send().await?;

    let (mut months, _) = client
        .get_months(plan.id())
        .with_server_knowledge(sk)
        .send()
        .await?;

    let month = months.pop().unwrap();
    println!("{}: {:?}", month.month, month.note);
    Ok(())
}
