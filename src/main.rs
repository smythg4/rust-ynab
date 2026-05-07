use rust_ynab::ynab::client::Client;
use rust_ynab::ynab::errors::Error;
#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_key = std::env::var("YNAB_TOKEN").expect("no api key found");
    let client = Client::new(api_key)?;

    let mut plans = client.get_plans(false).await?;


    let plan = plans.pop().unwrap();

    let months = client.get_months(plan.id, None).await?;
    for month in months {
        println!("{}: {}", month.month, month.income);
    }

    Ok(())
}
