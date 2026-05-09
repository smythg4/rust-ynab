use rust_ynab::Client;
use rust_ynab::PlanId;
use rust_ynab::milliunits_to_amount;

/// Finds the first eligible category and prints its budgeted, activity,
/// and balance for the plan's first month.
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

    let (mut cat_groups, _) = client.get_categories(plan_id).send().await?;
    cat_groups.retain(|cg| !cg.deleted && !cg.hidden && !cg.categories.is_empty());

    let category = cat_groups
        .iter()
        .flat_map(|cg| cg.categories.iter())
        .find(|c| !c.hidden && !c.deleted)
        .ok_or("no eligible category found")?;

    let month = plan.plan.first_month;
    let cat = client
        .get_category_for_month(plan_id, month, category.id)
        .await?;

    println!("Plan:     {}", plan.plan.name);
    println!("Month:    {}", month);
    println!("Category: {}\n", category.name);
    println!(
        "{:<12}  ${:>12.2}",
        "Budgeted",
        milliunits_to_amount(cat.budgeted)
    );
    println!(
        "{:<12}  ${:>12.2}",
        "Activity",
        milliunits_to_amount(cat.activity)
    );
    println!(
        "{:<12}  ${:>12.2}",
        "Balance",
        milliunits_to_amount(cat.balance)
    );

    Ok(())
}
