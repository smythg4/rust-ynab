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
struct SaveCategoryGroupDataEnvelope {
    data: CategoryGroupData,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoryGroupData {
    category_group: CategoryGroup,
    server_knowledge: i64,
}

/// A group of budget categories.
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryGroup {
    pub id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub deleted: bool,
    #[serde(default)]
    pub categories: Vec<Category>,
}

/// A budget category. Amounts (assigned, activity, available, etc.) are specific to the current
/// plan month (UTC) and are in milliunits (divide by 1000 for display).
#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub category_group_id: Uuid,
    pub category_group_name: Option<String>,
    pub name: String,
    pub hidden: bool,
    pub original_category_group_id: Option<Uuid>,
    pub note: Option<String>,
    pub budgeted: i64,
    pub activity: i64,
    pub balance: i64,
    pub goal_type: Option<GoalType>,
    pub goal_needs_whole_amount: Option<bool>,
    pub goal_day: Option<usize>,
    pub goal_cadence: Option<usize>,
    pub goal_cadence_frequency: Option<usize>,
    pub goal_creation_month: Option<NaiveDate>,
    pub goal_target: Option<i64>,
    pub goal_target_date: Option<NaiveDate>,
    pub goal_target_month: Option<NaiveDate>,
    pub goal_percentage_complete: Option<usize>,
    pub goal_months_to_budget: Option<usize>,
    pub goal_under_funded: Option<i64>,
    pub goal_overall_funded: Option<i64>,
    pub goal_overall_left: Option<i64>,
    pub goal_snoozed_at: Option<DateTime<chrono::Utc>>,
    pub deleted: bool,
}

/// The type of savings or spending goal assigned to a category.
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

#[derive(Debug)]
pub struct GetCategoriesBuilder<'a> {
    client: &'a Client,
    plan_id: PlanId,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetCategoriesBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> GetCategoriesBuilder<'a> {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    pub async fn send(self) -> Result<(Vec<CategoryGroup>, i64), Error> {
        let params: Option<&[(&str, &str)]> = if let Some(sk) = self.last_knowledge_of_server {
            Some(&[("last_knowledge_of_server", &sk.to_string())])
        } else {
            None
        };
        let result: CategoriesDataEnvelope = self
            .client
            .get(&format!("plans/{}/categories", self.plan_id), params)
            .await?;
        Ok((result.data.category_groups, result.data.server_knowledge))
    }
}

impl Client {
    /// Returns all categories grouped by category group. Amounts (assigned, activity, available,
    /// etc.) are specific to the current plan month (UTC). The second return value is server
    /// knowledge for delta requests.
    pub fn get_categories(&self, plan_id: PlanId) -> GetCategoriesBuilder<'_> {
        GetCategoriesBuilder {
            client: self,
            plan_id,
            last_knowledge_of_server: None,
        }
    }

    /// Returns a single category. Amounts (assigned, activity, available, etc.) are specific to
    /// the current plan month (UTC).
    pub async fn get_category(&self, plan_id: PlanId, cat_id: Uuid) -> Result<Category, Error> {
        let result: CategoryDataEnvelope = self
            .get(
                &format!("plans/{}/categories/{}", plan_id, cat_id),
                NO_PARAMS,
            )
            .await?;

        Ok(result.data.category)
    }

    /// Returns a single category for a specific plan month. Amounts (assigned, activity,
    /// available, etc.) are specific to the current plan month (UTC).
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

/// The category group to create or update.
#[derive(Debug, Serialize)]
pub struct SaveCategoryGroup {
    pub name: String,
}

/// The category to create.
#[derive(Debug, Serialize)]
pub struct NewCategory {
    pub name: String,
    pub category_group_id: Uuid,
    pub note: Option<String>,
    pub goal_target: Option<i64>,
    pub goal_target_date: Option<NaiveDate>,
    pub goal_needs_whole_amount: Option<bool>,
}

/// The category to update. Only specified (non-`None`) fields will be changed.
#[derive(Debug, Serialize)]
pub struct SaveCategory {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_group_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_target: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_target_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_needs_whole_amount: Option<bool>,
}

/// The month category to update. Only `budgeted` (assigned) can be changed.
#[derive(Debug, Serialize)]
pub struct SaveMonthCategory {
    budgeted: i64,
}

#[derive(Debug, Serialize)]
struct NewCategoryBody {
    category: NewCategory,
}

#[derive(Debug, Serialize)]
struct SaveCategoryBody {
    category: SaveCategory,
}

#[derive(Debug, Serialize)]
struct SaveMonthCategoryBody {
    category: SaveMonthCategory,
}

#[derive(Debug, Serialize)]
struct SaveCategoryGroupBody {
    category_group: SaveCategoryGroup,
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveCategoryDataEnvelope {
    data: SaveCategoryData,
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveCategoryData {
    category: Category,
    server_knowledge: i64,
}

impl Client {
    /// Creates a new category.
    pub async fn create_category(
        &self,
        plan_id: PlanId,
        category: NewCategory,
    ) -> Result<(Category, i64), Error> {
        let result: SaveCategoryDataEnvelope = self
            .post(
                &format!("plans/{plan_id}/categories"),
                NewCategoryBody { category },
            )
            .await?;
        Ok((result.data.category, result.data.server_knowledge))
    }

    /// Creates a new category group.
    pub async fn create_category_group(
        &self,
        plan_id: PlanId,
        category_group: SaveCategoryGroup,
    ) -> Result<(CategoryGroup, i64), Error> {
        let result: SaveCategoryGroupDataEnvelope = self
            .post(
                &format!("plans/{plan_id}/category_groups"),
                SaveCategoryGroupBody { category_group },
            )
            .await?;
        Ok((result.data.category_group, result.data.server_knowledge))
    }

    /// Update a category.
    pub async fn update_category(
        &self,
        plan_id: PlanId,
        category_id: Uuid,
        category: SaveCategory,
    ) -> Result<(Category, i64), Error> {
        let result: SaveCategoryDataEnvelope = self
            .patch(
                &format!("plans/{plan_id}/categories/{category_id}"),
                SaveCategoryBody { category },
            )
            .await?;
        Ok((result.data.category, result.data.server_knowledge))
    }

    /// Update a category for a specific month. Only `budgeted` (assigned) amount can be updated.`
    pub async fn update_category_for_month(
        &self,
        plan_id: PlanId,
        month: NaiveDate,
        category_id: Uuid,
        category: SaveMonthCategory,
    ) -> Result<(Category, i64), Error> {
        let result: SaveCategoryDataEnvelope = self
            .patch(
                &format!("plans/{plan_id}/months/{month}/categories/{category_id}"),
                SaveMonthCategoryBody { category },
            )
            .await?;
        Ok((result.data.category, result.data.server_knowledge))
    }

    /// Update a category group.
    pub async fn update_category_group(
        &self,
        plan_id: PlanId,
        category_group_id: Uuid,
        category_group: SaveCategoryGroup,
    ) -> Result<(CategoryGroup, i64), Error> {
        let result: SaveCategoryGroupDataEnvelope = self
            .patch(
                &format!("plans/{plan_id}/category_groups/{category_group_id}"),
                SaveCategoryGroupBody { category_group },
            )
            .await?;
        Ok((result.data.category_group, result.data.server_knowledge))
    }
}
