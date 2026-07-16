use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Client;
use crate::Error;
use crate::PlanId;
use crate::ynab::common::{NO_PARAMS, ServerKnowledge};

#[derive(Debug, Serialize, Deserialize)]
struct CategoriesDataEnvelope {
    data: CategoriesData,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoriesData {
    category_groups: Vec<CategoryGroup>,
    server_knowledge: ServerKnowledge,
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
    server_knowledge: ServerKnowledge,
}

/// A group of budget categories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
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
    last_knowledge_of_server: Option<ServerKnowledge>,
}

impl<'a> GetCategoriesBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: ServerKnowledge) -> GetCategoriesBuilder<'a> {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    /// Sends the request. Returns category groups (each containing their categories) and server knowledge for use in subsequent delta requests.
    pub async fn send(self) -> Result<(Vec<CategoryGroup>, ServerKnowledge), Error> {
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
    /// Returns a builder for fetching all categories grouped by category group. Chain
    /// `.with_server_knowledge()` for a delta request.
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
        month: NaiveDate,
        cat_id: Uuid,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SaveCategoryGroup {
    pub name: String,
}

/// The category to create.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NewCategory {
    pub name: String,
    pub category_group_id: Uuid,
    pub note: Option<String>,
    pub goal_target: Option<i64>,
    pub goal_target_date: Option<NaiveDate>,
    pub goal_needs_whole_amount: Option<bool>,
}

/// The category to update. Only specified (non-`None`) fields will be changed.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SaveMonthCategory {
    pub budgeted: i64,
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
    server_knowledge: ServerKnowledge,
}

impl Client {
    /// Creates a new category.
    pub async fn create_category(
        &self,
        plan_id: PlanId,
        category: NewCategory,
    ) -> Result<(Category, ServerKnowledge), Error> {
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
    ) -> Result<(CategoryGroup, ServerKnowledge), Error> {
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
    ) -> Result<(Category, ServerKnowledge), Error> {
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
    ) -> Result<(Category, ServerKnowledge), Error> {
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
    ) -> Result<(CategoryGroup, ServerKnowledge), Error> {
        let result: SaveCategoryGroupDataEnvelope = self
            .patch(
                &format!("plans/{plan_id}/category_groups/{category_group_id}"),
                SaveCategoryGroupBody { category_group },
            )
            .await?;
        Ok((result.data.category_group, result.data.server_knowledge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{
        TEST_ID_1, TEST_ID_2, category_fixture, category_group_fixture, error_body, new_test_client,
    };
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    #[tokio::test]
    async fn update_category_for_month_succeeds() {
        let (client, server) = new_test_client().await;
        let month = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let fixture = category_fixture();
        let envelope = json!({ "data": { "category": fixture, "server_knowledge": 5 } });
        Mock::given(method("PATCH"))
            .and(path(format!(
                "/plans/{}/months/{}/categories/{}",
                TEST_ID_1, month, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(envelope))
            .expect(1)
            .mount(&server)
            .await;
        let (category, sk) = client
            .update_category_for_month(
                PlanId::Id(TEST_ID_1.parse().unwrap()),
                month,
                TEST_ID_1.parse().unwrap(),
                SaveMonthCategory { budgeted: 75000 },
            )
            .await
            .unwrap();
        assert_eq!(category.id.to_string(), TEST_ID_1);
        assert_eq!(sk, 5);
    }

    #[tokio::test]
    async fn create_category_succeeds() {
        let (client, server) = new_test_client().await;

        let fixture = category_fixture();
        let envelope = json!({
            "data": {
                "category": fixture,
                "server_knowledge": 1
            }
        });

        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/categories", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(201).set_body_json(envelope))
            .expect(1)
            .mount(&server)
            .await;

        let category = NewCategory {
            name: fixture["name"].as_str().unwrap().to_string(),
            category_group_id: TEST_ID_2.parse().unwrap(),
            note: None,
            goal_target: None,
            goal_target_date: None,
            goal_needs_whole_amount: None,
        };

        let (response, sk) = client
            .create_category(PlanId::Id(TEST_ID_1.parse().unwrap()), category)
            .await
            .unwrap();

        assert_eq!(response.id.to_string(), TEST_ID_1);
        assert_eq!(response.name, fixture["name"].as_str().unwrap());
        assert_eq!(response.balance, fixture["balance"].as_i64().unwrap());
        assert_eq!(sk, 1);
    }

    #[tokio::test]
    async fn create_category_returns_internal_server_error() {
        let (client, server) = new_test_client().await;

        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/categories", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(500).set_body_json(error_body(
                "500",
                "internal_server_error",
                "An internal error occurred",
            )))
            .expect(1)
            .mount(&server)
            .await;

        let category = NewCategory {
            name: "Groceries".to_string(),
            category_group_id: TEST_ID_2.parse().unwrap(),
            note: None,
            goal_target: None,
            goal_target_date: None,
            goal_needs_whole_amount: None,
        };

        let result = client
            .create_category(PlanId::Id(TEST_ID_1.parse().unwrap()), category)
            .await;

        assert!(matches!(result, Err(Error::InternalServerError(_))));
    }

    #[tokio::test]
    async fn get_categories_returns_category_groups() {
        let (client, server) = new_test_client().await;
        let fixture = json!({
            "data": { "category_groups": [category_group_fixture()], "server_knowledge": 2 }
        });
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/categories", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture))
            .expect(1)
            .mount(&server)
            .await;
        let (groups, sk) = client
            .get_categories(PlanId::Id(TEST_ID_1.parse().unwrap()))
            .send()
            .await
            .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].id.to_string(), TEST_ID_2);
        assert_eq!(groups[0].categories.len(), 1);
        assert_eq!(sk, 2);
    }

    #[tokio::test]
    async fn get_category_returns_category() {
        let (client, server) = new_test_client().await;
        let fixture = category_fixture();
        let envelope = json!({ "data": { "category": fixture } });
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/categories/{}",
                TEST_ID_1, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(envelope))
            .expect(1)
            .mount(&server)
            .await;
        let category = client
            .get_category(
                PlanId::Id(TEST_ID_1.parse().unwrap()),
                TEST_ID_1.parse().unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(category.id.to_string(), TEST_ID_1);
        assert_eq!(category.name, "Groceries");
    }

    #[tokio::test]
    async fn get_category_for_month_returns_category() {
        let (client, server) = new_test_client().await;
        let month = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let fixture = category_fixture();
        let envelope = json!({ "data": { "category": fixture } });
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/months/{}/categories/{}",
                TEST_ID_1, month, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(envelope))
            .expect(1)
            .mount(&server)
            .await;
        let category = client
            .get_category_for_month(
                PlanId::Id(TEST_ID_1.parse().unwrap()),
                month,
                TEST_ID_1.parse().unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(category.id.to_string(), TEST_ID_1);
    }

    #[tokio::test]
    async fn create_category_group_succeeds() {
        let (client, server) = new_test_client().await;
        let fixture = category_group_fixture();
        let envelope = json!({ "data": { "category_group": fixture, "server_knowledge": 2 } });
        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/category_groups", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(201).set_body_json(envelope))
            .expect(1)
            .mount(&server)
            .await;
        let (group, sk) = client
            .create_category_group(
                PlanId::Id(TEST_ID_1.parse().unwrap()),
                SaveCategoryGroup {
                    name: "Everyday Expenses".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(group.id.to_string(), TEST_ID_2);
        assert_eq!(sk, 2);
    }

    #[tokio::test]
    async fn update_category_succeeds() {
        let (client, server) = new_test_client().await;
        let fixture = category_fixture();
        let envelope = json!({ "data": { "category": fixture, "server_knowledge": 4 } });
        Mock::given(method("PATCH"))
            .and(path(format!(
                "/plans/{}/categories/{}",
                TEST_ID_1, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(envelope))
            .expect(1)
            .mount(&server)
            .await;
        let (category, sk) = client
            .update_category(
                PlanId::Id(TEST_ID_1.parse().unwrap()),
                TEST_ID_1.parse().unwrap(),
                SaveCategory {
                    name: Some("Groceries".to_string()),
                    category_group_id: None,
                    note: None,
                    goal_target: None,
                    goal_target_date: None,
                    goal_needs_whole_amount: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(category.id.to_string(), TEST_ID_1);
        assert_eq!(sk, 4);
    }

    #[tokio::test]
    async fn update_category_group_succeeds() {
        let (client, server) = new_test_client().await;
        let fixture = category_group_fixture();
        let envelope = json!({ "data": { "category_group": fixture, "server_knowledge": 4 } });
        Mock::given(method("PATCH"))
            .and(path(format!(
                "/plans/{}/category_groups/{}",
                TEST_ID_1, TEST_ID_2
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(envelope))
            .expect(1)
            .mount(&server)
            .await;
        let (group, sk) = client
            .update_category_group(
                PlanId::Id(TEST_ID_1.parse().unwrap()),
                TEST_ID_2.parse().unwrap(),
                SaveCategoryGroup {
                    name: "Everyday Expenses".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(group.id.to_string(), TEST_ID_2);
        assert_eq!(sk, 4);
    }
}
