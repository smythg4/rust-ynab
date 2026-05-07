use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Client;
use crate::Error;
use crate::PlanId;
use crate::ynab::common::NO_PARAMS;

#[derive(Debug, Serialize, Deserialize)]
struct CategoriesDataEnvelope {
    data: CategoriesData,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoriesData {
    category_groups: Vec<CategoryGroup>,
    server_knowledge: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoryDataEnvelope {
    data: CategoryData,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoryData {
    category: Category,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoryGroupDataEnvelope {
    data: CategoryGroupData,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoryGroupData {
    category_groups: CategoryGroup,
    server_knowldge: i64,
}

/// CategoryGroup represents a group of budget categories.
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryGroup {
    id: Uuid,
    name: String,
    hidden: bool,
    deleted: bool,
    categories: Vec<Category>,
}

/// Category represents a single budget category with goal and balance information.
/// Amounts are in milliunits (divide by 1000 for display).
#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    id: Uuid,
    category_group_id: Uuid,
    category_group_name: String,
    name: String,
    hidden: bool,
    original_category_group_id: Option<Uuid>,
    note: Option<String>,
    budgeted: i64,
    activity: i64,
    balance: i64,
    goal_type: Option<GoalType>,
    goal_needs_whole_amount: Option<bool>,
    goal_day: Option<usize>,
    goal_cadence: Option<usize>,
    goal_cadence_frequency: Option<usize>,
    goal_creation_month: Option<NaiveDate>,
    goal_target: Option<i64>,
    goal_target_date: Option<NaiveDate>,
    goal_percentage_complete: Option<usize>,
    goal_months_to_budget: Option<usize>,
    goal_under_funded: Option<i64>,
    goal_overall_funded: Option<i64>,
    goal_overall_left: Option<i64>,
    goal_snoozed_at: Option<DateTime<chrono::Utc>>,
    deleted: bool,
}

/// GoalType represents the type of savings or spending goal assigned to a category.
#[derive(Debug, Serialize, Deserialize)]
pub enum GoalType {
    #[serde(rename = "TB")]
    TargetBalance, // "TB"
    #[serde(rename = "TBD")]
    TargetBalanceByDate, // "TBD"
    #[serde(rename = "NEED")]
    PlanYourSpending, // "NEED"
    #[serde(rename = "MF")]
    MonthlyFunding, // "MF"
    #[serde(rename = "DEBT")]
    Debt, // "DEBT"
    #[serde(other)]
    Other,
}

impl Client {
    /// get_categories returns all category groups and their categories for a plan.
    /// The second return value is server knowledge for delta requests.
    pub async fn get_categories(
        &self,
        plan_id: PlanId,
        last_server_knowledge: Option<i64>,
    ) -> Result<(Vec<CategoryGroup>, i64), Error> {
        let sk_owned = last_server_knowledge.map(|sk| sk.to_string());
        let params: Vec<(&str, &str)> = sk_owned
            .as_deref()
            .map(|sk| vec![("last_knowledge_of_server", sk)])
            .unwrap_or_default();

        let result: CategoriesDataEnvelope = self
            .get(&format!("plans/{}/categories", plan_id), &params)
            .await?;
        Ok((result.data.category_groups, result.data.server_knowledge))
    }

    /// get_category returns a single category by ID.
    pub async fn get_category(&self, plan_id: PlanId, cat_id: Uuid) -> Result<Category, Error> {
        let result: CategoryDataEnvelope = self
            .get(&format!("plans/{}/categories/{}", plan_id, cat_id), NO_PARAMS)
            .await?;

        Ok(result.data.category)
    }

    /// get_category_for_month returns a category's data for a specific budget month.
    pub async fn get_category_for_month(
        &self,
        plan_id: PlanId,
        cat_id: Uuid,
        month: NaiveDate,
    ) -> Result<Category, Error> {
        let result: CategoryDataEnvelope = self
            .get(
                &format!("plans/{}/months/{}/categories/{}", plan_id, month, cat_id),
                NO_PARAMS,
            )
            .await?;

        Ok(result.data.category)
    }
}
