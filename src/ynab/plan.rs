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

#[derive(Debug)]
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
    default_plan: Option<Plan>,
}

/// Plan represents a YNAB budget plan.
#[derive(Debug, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub name: String,
    pub last_modified_on: DateTime<chrono::Utc>,
    pub first_month: NaiveDate,
    pub last_month: NaiveDate,
    pub date_format: DateFormat,
    pub currency_format: CurrencyFormat,
    pub accounts: Option<Vec<Account>>,
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

// PlanDetails is the full plan export returned by GetPlan, including all
// accounts, categories, transactions, and other sub-resources.
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
    /// get_plans returns all plans for the authenticated user. include_accounts flag indicates
    /// whether you want the returned payload to include all the account information for each
    /// plan.
    pub fn get_plans(&self) -> GetPlansBuilder<'_> {
        GetPlansBuilder {
            client: self,
            include_accounts: false,
        }
    }

    /// get_plan_settings returns the date and currency format settings for a plan.
    pub async fn get_plan_settings(&self, plan_id: PlanId) -> Result<PlanSettings, Error> {
        let result: PlanSettingsDataEnvelope = self
            .get(&format!("plans/{}/settings", plan_id), NO_PARAMS)
            .await?;
        Ok(result.data.settings)
    }

    /// get_plan returns the full export for the given plan, including all
    /// sub-resources. The second return value is server knowledge for delta requests.
    /// For large plans this response can be substantial —
    /// consider using specific resource endpoints for targeted queries.
    pub fn get_plan(&self, plan_id: PlanId) -> GetPlanBuilder<'_> {
        GetPlanBuilder {
            plan_id,
            client: self,
            last_knowledge_of_server: None,
        }
    }
}
