use chrono::Datelike;
use rust_ynab::Client;
use rust_ynab::PlanId;
use rust_ynab::milliunits_to_amount;
use rust_ynab::ynab::category::SaveMonthCategory;

/// Fetches the first eligible category, increases its budget for the current month
/// by $10, and prints the before and after state.
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
    cat_groups.retain(|cg| !cg.hidden && !cg.deleted && cg.name != "Internal Master Category");

    let category = cat_groups
        .iter()
        .flat_map(|cg| cg.categories.iter())
        .find(|c| !c.hidden && !c.deleted)
        .ok_or("no eligible categories found")?;

    let (cat_id, cat_name) = (category.id, category.name.clone());

    let month = chrono::Local::now().date_naive().with_day(1).unwrap();

    let current = client
        .get_category_for_month(plan_id, month, cat_id)
        .await?;

    let (updated, _) = client
        .update_category_for_month(
            plan_id,
            month,
            cat_id,
            SaveMonthCategory {
                budgeted: current.budgeted + 10_000,
            },
        )
        .await?;

    println!("Updated Category Budget\n");
    println!("Plan:  {}", plan.plan.name);
    println!("Month: {}\n", month);
    println!("   {:<12} {}", "Category:", cat_name);
    println!(
        "   {:<12} ${:.2}  ->  ${:.2}",
        "Budgeted:",
        milliunits_to_amount(current.budgeted),
        milliunits_to_amount(updated.budgeted),
    );

    Ok(())
}
