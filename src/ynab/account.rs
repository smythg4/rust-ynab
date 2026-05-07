use chrono::DateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Client;
use crate::Error;
use crate::PlanId;
use crate::ynab::common::NO_PARAMS;

#[derive(Debug, Deserialize, Serialize)]
struct AccountDataEnvelope {
    data: AccountData,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccountData {
    account: Account,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccountsDataEnvelope {
    data: AccountsData,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccountsData {
    accounts: Vec<Account>,
    server_knowledge: i64,
}

/// AccountType represents the type of a YNAB account.
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

/// Account represents a YNAB account such as checking, savings, or credit card.
#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: Uuid,
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

#[derive(Debug)]
pub struct GetAccountsBuilder<'a> {
    client: &'a Client,
    plan_id: PlanId,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetAccountsBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> GetAccountsBuilder<'a> {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    pub async fn send(self) -> Result<(Vec<Account>, i64), Error> {
        let params: Option<&[(&str, &str)]> = if let Some(sk) = self.last_knowledge_of_server {
            Some(&[("last_knowledge_of_server", &sk.to_string())])
        } else {
            None
        };
        let result: AccountsDataEnvelope = self
            .client
            .get(&format!("plans/{}/accounts", self.plan_id), params)
            .await?;
        Ok((result.data.accounts, result.data.server_knowledge))
    }
}

impl Client {
    /// get_accounts returns all accounts for a plan. The second return value is server knowledge for delta requests.
    pub fn get_accounts(&self, plan_id: PlanId) -> GetAccountsBuilder<'_> {
        GetAccountsBuilder {
            client: self,
            plan_id,
            last_knowledge_of_server: None,
        }
    }

    /// get_account returns a single account by ID.
    pub async fn get_account(&self, plan_id: PlanId, account_id: Uuid) -> Result<Account, Error> {
        let result: AccountDataEnvelope = self
            .get(
                &format!("plans/{}/accounts/{}", plan_id, account_id),
                NO_PARAMS,
            )
            .await?;
        Ok(result.data.account)
    }
}
