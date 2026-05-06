use rust_ynab::ynab::client::Client;
use rust_ynab::ynab::errors::Error;
#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_key = std::env::var("YNAB_TOKEN").expect("no api key found");
    let client = Client::new(api_key)?;

    let plans = client.get_plans(false).await?;

    for plan in plans {
        println!("{}", plan.name);
        let settings = client.get_plan_settings(plan.id).await?;
        println!("   {:?}", settings.currency_format);
    }

    Ok(())
}
