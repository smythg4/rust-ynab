use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Client;
use crate::Error;
use crate::Month;
use crate::{Category, CategoryGroup};
use crate::{Payee, PayeeLocation};
use crate::{ScheduledSubTransation, ScheduledTransaction, Subtransaction, Transaction};

// TODO: Move accounts to separate module
#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: uuid::Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub acct_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    pub balance: i64,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub transfer_payee_id: Option<uuid::Uuid>,
    pub direct_import_linked: bool,
    pub direct_import_in_error: bool,
    pub last_reconciled_at: Option<DateTime<chrono::Utc>>,
    pub deleted: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountType {
    Checking,
    Savings,
    Cash,
    CreditCard,
    OtherAsset,
    OtherLiability,
    StudentLoan,
    #[serde(other)]
    Other,
}
// End account structs

// TODO: Move into common module
#[derive(Debug, Deserialize, Serialize)]
pub struct DateFormat {
    format: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrencyFormat {
    iso_code: String,
    example_format: String,
    decimal_digits: usize,
    decimal_separator: char,
    symbol_first: bool,
    group_separator: String,
    currency_symbol: String,
    display_symbol: bool,
}
// End formats for common module

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
    pub scheduled_subtransactions: Vec<ScheduledSubTransation>,
}

impl Client {
    /// get_plans returns all plans for the authenticated user. include_accounts flag indicates
    /// whether you want the returned payload to include all the account information for each
    /// plan.
    pub async fn get_plans(&self, include_accounts: bool) -> Result<Vec<Plan>, Error> {
        let mut params = vec![];
        if include_accounts {
            params.push(("include_accounts", "true"));
        }
        let result: PlanDataEnvelope = self.get("plans", &params).await?;
        Ok(result.data.plans)
    }

    /// get_plan_settings returns the date and currency format settings for a plan.
    pub async fn get_plan_settings(&self, plan_id: Uuid) -> Result<PlanSettings, Error> {
        let result: PlanSettingsDataEnvelope = self
            .get(&format!("plans/{}/settings", plan_id), &[])
            .await?;
        Ok(result.data.settings)
    }

    /// get_plan returns the full export for the given plan, including all
    /// sub-resources. The second return value is server knowledge for delta requests.
    /// For large plans this response can be substantial —
    /// consider using specific resource endpoints for targeted queries.
    pub async fn get_plan(
        &self,
        plan_id: Uuid,
        params: &[bool],
    ) -> Result<(PlanDetails, i64), Error> {
        let result: PlanDetailsDataEnvelope = self.get(&format!("plans/{}", plan_id), &[]).await?;
        Ok((result.data.plan, result.data.server_knowledge))
    }

    /// get_last_plan_used returns the full export for the most recently used plan
    /// for the authenticated user. Use the returned plan's ID for subsequent
    /// sub-resource calls (GetAccounts, GetTransactions, etc.) — there is no
    /// "last-used" shortcut for sub-resource endpoints.
    pub async fn get_last_plan_used(&self, params: &[bool]) -> Result<(PlanDetails, i64), Error> {
        let result: PlanDetailsDataEnvelope = self.get("plans/last-used", &[]).await?;
        Ok((result.data.plan, result.data.server_knowledge))
    }
}
