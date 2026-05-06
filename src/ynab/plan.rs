use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ynab::client::Client;
use crate::ynab::errors::Error;

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
/* not ready yet

#[derive(Debug, Serialize, Deserialize)]
struct PlanDetailsDataEnvelope {
    data: PlanDetailsData,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlanDetailsData {
    plan: PlanDetails,
    server_knowledge: i64,
}

/// PlanDetails is the full plan export returned by GetPlan, including all
/// accounts, categories, transactions, and other sub-resources.
// #[derive(Debug, Serialize, Deserialize)]
pub struct PlanDetails {
    #[serde(flatten)]
    plan: Plan,
    payees: Vec<Payee>,
    payee_locations: Vec<PayeeLocation>,
    category_groups: Vec<CategoryGroup>,
    categories: Vec<Category>,
    months: Vec<Month>,
    transactions: Vec<Transaction>,
    subtransactions: Vec<Subtransaction>,
    scheduled_transactions: Vec<ScheduledTransaction>,
    scheduled_subtransactions: Vec<ScheduledSubTransation>,
}
*/

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
            .get(&format!("plans/{}/settings", plan_id), &vec![])
            .await?;
        Ok(result.data.settings)
    }
}
