use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Account;
use crate::Client;
use crate::Error;
use crate::Month;
use crate::ynab::common::NO_PARAMS;
use crate::{Category, CategoryGroup};
use crate::{CurrencyFormat, DateFormat};
use crate::{Payee, PayeeLocation};
use crate::{ScheduledSubtransaction, ScheduledTransaction, Subtransaction, Transaction};

#[derive(Debug, Clone, Copy)]
pub enum PlanId {
    Id(Uuid),
    LastUsed,
    Default,
}

impl std::fmt::Display for PlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => write!(f, "{id}"),
            Self::LastUsed => write!(f, "last-used"),
            Self::Default => write!(f, "default"),
        }
    }
}

impl From<Uuid> for PlanId {
    fn from(value: Uuid) -> Self {
        Self::Id(value)
    }
}
#[derive(Debug, Deserialize)]
struct PlanDataEnvelope {
    data: PlanData,
}

#[derive(Debug, Deserialize)]
struct PlanData {
    plans: Vec<Plan>,
    // users can use PlanId::Default to directly interact with the default plan
    _default_plan: Option<Plan>,
}

/// Summary information for a plan.
#[derive(Debug, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub name: String,
    pub last_modified_on: DateTime<chrono::Utc>,
    pub first_month: NaiveDate,
    pub last_month: NaiveDate,
    pub date_format: DateFormat,
    pub currency_format: CurrencyFormat,
    #[serde(default)]
    pub accounts: Vec<Account>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlanSettingsDataEnvelope {
    data: PlanSettingsData,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlanSettingsData {
    settings: PlanSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanSettings {
    pub date_format: DateFormat,
    pub currency_format: CurrencyFormat,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlanDetailsDataEnvelope {
    data: PlanDetailsData,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlanDetailsData {
    plan: PlanDetails,
    server_knowledge: i64,
}

/// A single plan with all related entities. This resource is effectively a full plan export.
#[derive(Debug, Serialize, Deserialize)]
pub struct PlanDetails {
    #[serde(flatten)]
    pub plan: Plan,
    pub payees: Vec<Payee>,
    pub payee_locations: Vec<PayeeLocation>,
    pub category_groups: Vec<CategoryGroup>,
    pub categories: Vec<Category>,
    pub months: Vec<Month>,
    pub transactions: Vec<Transaction>,
    pub subtransactions: Vec<Subtransaction>,
    pub scheduled_transactions: Vec<ScheduledTransaction>,
    pub scheduled_subtransactions: Vec<ScheduledSubtransaction>,
}

impl PlanDetails {
    pub fn id(&self) -> PlanId {
        PlanId::Id(self.plan.id)
    }
}

#[derive(Debug)]
pub struct GetPlansBuilder<'a> {
    client: &'a Client,
    include_accounts: bool,
}

impl<'a> GetPlansBuilder<'a> {
    pub fn include_accounts(mut self) -> GetPlansBuilder<'a> {
        self.include_accounts = true;
        self
    }

    pub async fn send(self) -> Result<Vec<Plan>, Error> {
        let params: Option<&[(&str, &str)]> = if self.include_accounts {
            Some(&[("include_accounts", "true")])
        } else {
            None
        };
        let result: PlanDataEnvelope = self.client.get("plans", params).await?;
        Ok(result.data.plans)
    }
}

#[derive(Debug)]
pub struct GetPlanBuilder<'a> {
    client: &'a Client,
    plan_id: PlanId,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetPlanBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> GetPlanBuilder<'a> {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    pub async fn send(self) -> Result<(PlanDetails, i64), Error> {
        let params: Option<&[(&str, &str)]> = if let Some(sk) = self.last_knowledge_of_server {
            Some(&[("last_knowledge_of_server", &sk.to_string())])
        } else {
            None
        };
        let result: PlanDetailsDataEnvelope = self
            .client
            .get(&format!("plans/{}", self.plan_id), params)
            .await?;
        Ok((result.data.plan, result.data.server_knowledge))
    }
}
impl Client {
    /// Returns plans list with summary information.
    pub fn get_plans(&self) -> GetPlansBuilder<'_> {
        GetPlansBuilder {
            client: self,
            include_accounts: false,
        }
    }

    /// Returns settings for a plan.
    pub async fn get_plan_settings(&self, plan_id: PlanId) -> Result<PlanSettings, Error> {
        let result: PlanSettingsDataEnvelope = self
            .get(&format!("plans/{}/settings", plan_id), NO_PARAMS)
            .await?;
        Ok(result.data.settings)
    }

    /// Returns a single plan with all related entities. This resource is effectively a full plan
    /// export. The second return value is server knowledge for delta requests.
    pub fn get_plan(&self, plan_id: PlanId) -> GetPlanBuilder<'_> {
        GetPlanBuilder {
            plan_id,
            client: self,
            last_knowledge_of_server: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{
        TEST_ID_5, error_body, new_test_client, plan_details_fixture, plan_fixture,
    };
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn plans_list_fixture() -> serde_json::Value {
        json!({ "data": { "plans": [plan_fixture()], "_default_plan": null } })
    }

    fn plan_single_fixture() -> serde_json::Value {
        json!({ "data": { "plan": plan_details_fixture(), "server_knowledge": 5 } })
    }

    fn plan_settings_fixture() -> serde_json::Value {
        json!({
            "data": {
                "settings": {
                    "date_format": { "format": "MM/DD/YYYY" },
                    "currency_format": {
                        "iso_code": "USD", "example_format": "123,456.78", "decimal_digits": 2,
                        "decimal_separator": ".", "symbol_first": true, "group_separator": ",",
                        "currency_symbol": "$", "display_symbol": true
                    }
                }
            }
        })
    }

    #[tokio::test]
    async fn get_plans_returns_plans() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path("/plans"))
            .respond_with(ResponseTemplate::new(200).set_body_json(plans_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let plans = client.get_plans().send().await.unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].id.to_string(), TEST_ID_5);
        assert_eq!(plans[0].name, "My Budget");
    }

    #[tokio::test]
    async fn get_plan_returns_plan_and_server_knowledge() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path("/plans/last-used"))
            .respond_with(ResponseTemplate::new(200).set_body_json(plan_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (plan, sk) = client.get_plan(PlanId::LastUsed).send().await.unwrap();
        assert_eq!(plan.plan.id.to_string(), TEST_ID_5);
        assert_eq!(plan.plan.name, "My Budget");
        assert_eq!(sk, 5);
    }

    #[tokio::test]
    async fn get_plan_settings_returns_settings() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path("/plans/last-used/settings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(plan_settings_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let settings = client.get_plan_settings(PlanId::LastUsed).await.unwrap();
        assert_eq!(settings.currency_format.iso_code, "USD");
    }

    #[tokio::test]
    async fn get_plan_returns_unauthorized() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path("/plans/last-used"))
            .respond_with(ResponseTemplate::new(401).set_body_json(error_body(
                "401",
                "unauthorized",
                "Unauthorized",
            )))
            .mount(&server)
            .await;
        let err = client.get_plan(PlanId::LastUsed).send().await.unwrap_err();
        assert!(matches!(err, Error::Unauthorized(_)));
    }
}
