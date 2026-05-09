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

/// The type of account.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountType {
    Checking,
    Savings,
    Cash,
    CreditCard,
    LineOfCredit,
    OtherAsset,
    OtherLiability,
    Mortgage,
    AutoLoan,
    StudentLoan,
    PersonalLoan,
    MedicalDebt,
    OtherDebt,
}

/// A plan account. Amounts are in milliunits (divide by 1000 for display).
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
    /// Returns all accounts. The second return value is server knowledge for delta requests.
    pub fn get_accounts(&self, plan_id: PlanId) -> GetAccountsBuilder<'_> {
        GetAccountsBuilder {
            client: self,
            plan_id,
            last_knowledge_of_server: None,
        }
    }

    /// Returns a single account.
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

/// The type of account to create or update.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SaveAccountType {
    Checking,
    Savings,
    Cash,
    CreditCard,
    LineOfCredit,
    OtherAsset,
    OtherLiability,
    Mortgage,
    AutoLoan,
    StudentLoan,
    PersonalLoan,
    MedicalDebt,
    OtherDebt,
}

impl TryFrom<&str> for SaveAccountType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "checking" => Ok(SaveAccountType::Checking),
            "savings" => Ok(SaveAccountType::Savings),
            "cash" => Ok(SaveAccountType::Cash),
            "creditCard" => Ok(SaveAccountType::CreditCard),
            "lineOfCredit" => Ok(SaveAccountType::LineOfCredit),
            "otherAsset" => Ok(SaveAccountType::OtherAsset),
            "otherLiability" => Ok(SaveAccountType::OtherLiability),
            "mortgage" => Ok(SaveAccountType::Mortgage),
            "autoLoan" => Ok(SaveAccountType::AutoLoan),
            "studentLoan" => Ok(SaveAccountType::StudentLoan),
            "personalLoan" => Ok(SaveAccountType::PersonalLoan),
            "medicalDebt" => Ok(SaveAccountType::MedicalDebt),
            "otherDebt" => Ok(SaveAccountType::OtherDebt),
            _ => Err(format!("unknown account type: {}", value)),
        }
    }
}

/// The account to create.
#[derive(Debug, Serialize)]
pub struct SaveAccount {
    pub name: String,
    #[serde(rename = "type")]
    pub acct_type: SaveAccountType,
    pub balance: i64,
}

#[derive(Debug, Serialize)]
struct SaveAccountBody {
    account: SaveAccount,
}

impl Client {
    /// Creates a new account.
    pub async fn create_account(
        &self,
        plan_id: PlanId,
        account: SaveAccount,
    ) -> Result<Account, Error> {
        let response: AccountDataEnvelope = self
            .post(
                &format!("plans/{}/accounts", plan_id),
                SaveAccountBody { account },
            )
            .await?;
        Ok(response.data.account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{TEST_ID_1, account_fixture, error_body, new_test_client};
    use uuid::uuid;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn account_list_fixture() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "accounts": [account_fixture(), account_fixture()],
                "server_knowledge": 7
            }
        })
    }

    fn account_single_fixture() -> serde_json::Value {
        serde_json::json!({
            "data": { "account": account_fixture() }
        })
    }

    #[tokio::test]
    async fn get_accounts_returns_ids() {
        let (client, server) = new_test_client().await;

        Mock::given(method("GET"))
            .and(path("/plans/last-used/accounts"))
            .respond_with(ResponseTemplate::new(200).set_body_json(account_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;

        let (accounts, _) = client.get_accounts(PlanId::LastUsed).send().await.unwrap();
        assert_eq!(accounts.len(), 2);
        assert!(
            accounts
                .iter()
                .zip([TEST_ID_1, TEST_ID_1])
                .all(|(a, id)| a.id.to_string() == id)
        );
    }

    #[tokio::test]
    async fn get_account_returns_id() {
        let (client, server) = new_test_client().await;

        Mock::given(method("GET"))
            .and(path(format!("/plans/last-used/accounts/{}", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(account_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;

        let account = client
            .get_account(PlanId::LastUsed, uuid!(TEST_ID_1))
            .await
            .unwrap();
        assert_eq!(account.id.to_string(), TEST_ID_1);
    }

    #[tokio::test]
    async fn get_account_returns_not_found() {
        let (client, server) = new_test_client().await;

        Mock::given(method("GET"))
            .and(path(format!("/plans/last-used/accounts/{}", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(404).set_body_json(error_body(
                "404",
                "not_found",
                "Account not found",
            )))
            .mount(&server)
            .await;

        let err = client
            .get_account(PlanId::LastUsed, TEST_ID_1.parse().unwrap())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[tokio::test]
    async fn create_account_succeeds() {
        let (client, server) = new_test_client().await;

        let input_account = account_fixture();
        let account = SaveAccount {
            name: input_account["name"].as_str().unwrap().to_string(),
            acct_type: SaveAccountType::try_from(input_account["type"].as_str().unwrap()).unwrap(),
            balance: input_account["balance"].as_i64().unwrap(),
        };

        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/accounts", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(account_single_fixture()))
            .mount(&server)
            .await;

        let account_response = client
            .create_account(PlanId::Id(uuid!(TEST_ID_1)), account)
            .await
            .unwrap();
        assert_eq!(account_response.id.to_string(), TEST_ID_1);
        assert_eq!(
            account_response.name,
            input_account["name"].as_str().unwrap()
        );
        assert_eq!(
            account_response.balance,
            input_account["balance"].as_i64().unwrap()
        );
        assert_eq!(
            account_response.deleted,
            input_account["deleted"].as_bool().unwrap()
        );
    }

    #[tokio::test]
    async fn create_account_returns_bad_request() {
        let (client, server) = new_test_client().await;

        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/accounts", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(400).set_body_json(error_body(
                "400",
                "bad_request",
                "Bad Request",
            )))
            .mount(&server)
            .await;

        let account = SaveAccount {
            name: "A bad bad name".to_string(),
            acct_type: SaveAccountType::Cash,
            balance: -500,
        };
        let err = client
            .create_account(PlanId::Id(uuid!(TEST_ID_1)), account)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::BadRequest(_)));
    }

    #[tokio::test]
    async fn create_account_returns_conflict() {
        let (client, server) = new_test_client().await;

        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/accounts", TEST_ID_1)))
            .respond_with(
                ResponseTemplate::new(409).set_body_json(error_body("409", "conflict", "Conflict")),
            )
            .mount(&server)
            .await;

        let account = SaveAccount {
            name: "A conflicting conflicting name".to_string(),
            acct_type: SaveAccountType::Cash,
            balance: -500,
        };
        let err = client
            .create_account(PlanId::Id(uuid!(TEST_ID_1)), account)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Conflict(_)));
    }
}
