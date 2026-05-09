use rust_ynab::Client;
use rust_ynab::PlanId;
use rust_ynab::milliunits_to_amount;

/// Fetches budget totals (income, budgeted, activity) for the test plan's last month.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("YNAB_TOKEN")?;
    let client = Client::new(&token)?;

    let plan_id_str = std::env::var("YNAB_TEST_PLAN_ID")?;
    let (plan_details, _) = client
        .get_plan(PlanId::Id(plan_id_str.parse()?))
        .send()
        .await?;

    let month = client
        .get_month(plan_details.id(), plan_details.plan.last_month)
        .await?;

    println!("Plan: {}", plan_details.plan.name);
    println!(
        "{:<12}  {:>13}  {:>13}  {:>13}",
        "Month", "Income", "Budgeted", "Activity"
    );
    println!(
        "{:<12}  {:>13}  {:>13}  {:>13}",
        "------------", "-------------", "-------------", "-------------"
    );
    println!(
        "{:<12}  ${:>12.2}  ${:>12.2}  ${:>12.2}",
        month.month,
        milliunits_to_amount(month.income),
        milliunits_to_amount(month.budgeted),
        milliunits_to_amount(month.activity),
    );
    Ok(())
}
