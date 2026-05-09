use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::PlanId;
use crate::ynab::client::Client;
use crate::ynab::common::NO_PARAMS;
use crate::ynab::errors::Error;

// --- Envelopes ---

#[derive(Debug, Deserialize)]
struct TransactionDataEnvelope {
    data: TransactionData,
}

#[derive(Debug, Deserialize)]
struct TransactionData {
    transaction: Transaction,
    server_knowledge: i64,
}

#[derive(Debug, Deserialize)]
struct TransactionsDataEnvelope {
    data: TransactionsData,
}

#[derive(Debug, Deserialize)]
struct TransactionsData {
    transactions: Vec<Transaction>,
    server_knowledge: i64,
}

#[derive(Debug, Deserialize)]
struct ScheduledTransactionDataEnvelope {
    data: ScheduledTransactionData,
}

#[derive(Debug, Deserialize)]
struct ScheduledTransactionData {
    scheduled_transaction: ScheduledTransaction,
}

#[derive(Debug, Deserialize)]
struct ScheduledTransactionsDataEnvelope {
    data: ScheduledTransactionsData,
}

#[derive(Debug, Deserialize)]
struct ScheduledTransactionsData {
    scheduled_transactions: Vec<ScheduledTransaction>,
    server_knowledge: i64,
}

#[derive(Debug, Deserialize)]
struct SaveTransactionsDataEnvelope {
    data: SaveTransactionsResponse,
}

/// Response from creating or batch-updating transactions.
#[derive(Debug, Deserialize)]
pub struct SaveTransactionsResponse {
    pub transaction_ids: Vec<Uuid>,
    pub transaction: Option<Transaction>,
    pub transactions: Option<Vec<Transaction>>,
    pub duplicate_import_ids: Option<Vec<Uuid>>,
    pub server_knowledge: i64,
}

// --- Enums ---

/// The cleared status of a transaction.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClearedStatus {
    Cleared,
    Uncleared,
    Reconciled,
}

/// The color of a transaction flag.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagColor {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}

/// The recurrence frequency of a scheduled transaction.
#[derive(Debug, Serialize, Deserialize)]
pub enum Frequency {
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
    #[serde(rename = "everyOtherWeek")]
    EveryOtherWeek,
    #[serde(rename = "twiceAMonth")]
    TwiceAMonth,
    #[serde(rename = "every4Weeks")]
    Every4Weeks,
    #[serde(rename = "monthly")]
    Monthly,
    #[serde(rename = "everyOtherMonth")]
    EveryOtherMonth,
    #[serde(rename = "every3Months")]
    Every3Months,
    #[serde(rename = "every4Months")]
    Every4Months,
    #[serde(rename = "twiceAYear")]
    TwiceAYear,
    #[serde(rename = "yearly")]
    Yearly,
    #[serde(rename = "everyOtherYear")]
    EveryOtherYear,
}

/// A plan transaction, excluding any pending transactions. Amounts are in milliunits (divide by
/// 1000 for display).
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub date: NaiveDate,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedStatus,
    pub approved: bool,
    pub flag_color: Option<FlagColor>,
    pub flag_name: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub account_name: Option<String>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub matched_transaction_id: Option<String>,
    pub import_id: Option<String>,
    pub import_payee_name: Option<String>,
    pub import_payee_name_original: Option<String>,
    pub deleted: bool,
    #[serde(default)]
    pub subtransactions: Vec<Subtransaction>,
}

/// A line item within a split transaction. Amounts are in milliunits (divide by 1000 for display).
#[derive(Debug, Serialize, Deserialize)]
pub struct Subtransaction {
    pub id: String,
    pub transaction_id: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<String>,
}

/// A scheduled transaction. Amounts are in milliunits (divide by 1000 for display).
#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduledTransaction {
    pub id: Uuid,
    pub date_first: NaiveDate,
    pub date_next: NaiveDate,
    pub frequency: Frequency,
    pub amount: i64,
    pub memo: Option<String>,
    pub flag_color: Option<FlagColor>,
    pub flag_name: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<ScheduledSubtransaction>,
    pub transfer_account_id: Option<Uuid>,
}

/// A line item within a split scheduled transaction. Amounts are in milliunits (divide by 1000 for
/// display).
#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduledSubtransaction {
    pub id: Uuid,
    pub scheduled_transaction_id: Uuid,
    pub amount: i64,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Debug)]
pub enum TransactionType {
    Uncategorized,
    Unapproved,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unapproved => write!(f, "unapproved"),
            Self::Uncategorized => write!(f, "uncategorized"),
        }
    }
}

#[derive(Debug)]
enum TransactionScope {
    All,
    ByAccount(Uuid),
    ByCategory(Uuid),
    ByPayee(Uuid),
    ByMonth(NaiveDate),
}
#[derive(Debug)]
pub struct GetTransactionsBuilder<'a> {
    client: &'a Client,
    scope: TransactionScope,
    plan_id: PlanId,
    since_date: Option<NaiveDate>,
    transaction_type: Option<TransactionType>,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetTransactionsBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> Self {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    pub fn since_date(mut self, since_date: NaiveDate) -> Self {
        self.since_date = Some(since_date);
        self
    }

    pub fn transaction_type(mut self, tx_type: TransactionType) -> Self {
        self.transaction_type = Some(tx_type);
        self
    }

    pub async fn send(self) -> Result<(Vec<Transaction>, i64), Error> {
        let date_str = self.since_date.map(|d| d.to_string());
        let type_str = self.transaction_type.map(|t| t.to_string());
        let sk_str = self.last_knowledge_of_server.map(|sk| sk.to_string());

        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(ref s) = date_str {
            params.push(("since_date", s));
        }
        if let Some(ref t) = type_str {
            params.push(("type", t));
        }
        if let Some(ref s) = sk_str {
            params.push(("last_knowledge_of_server", s));
        }
        let url = match self.scope {
            TransactionScope::All => format!("plans/{}/transactions", self.plan_id),
            TransactionScope::ByAccount(id) => {
                format!("plans/{}/accounts/{}/transactions", self.plan_id, id)
            }
            TransactionScope::ByCategory(id) => {
                format!("plans/{}/categories/{}/transactions", self.plan_id, id)
            }
            TransactionScope::ByPayee(id) => {
                format!("plans/{}/payees/{}/transactions", self.plan_id, id)
            }
            TransactionScope::ByMonth(month) => {
                format!("plans/{}/months/{}/transactions", self.plan_id, month)
            }
        };
        let result: TransactionsDataEnvelope = self.client.get(&url, Some(&params)).await?;
        Ok((result.data.transactions, result.data.server_knowledge))
    }
}

impl Client {
    /// Returns plan transactions, excluding any pending transactions. The second return value is
    /// server knowledge for delta requests.
    pub fn get_transactions(&self, plan_id: PlanId) -> GetTransactionsBuilder<'_> {
        GetTransactionsBuilder {
            client: self,
            scope: TransactionScope::All,
            plan_id,
            since_date: None,
            transaction_type: None,
            last_knowledge_of_server: None,
        }
    }

    /// Returns a single transaction.
    pub async fn get_transaction(
        &self,
        plan_id: PlanId,
        transaction_id: &Uuid,
    ) -> Result<(Transaction, i64), Error> {
        let result: TransactionDataEnvelope = self
            .get(
                &format!("plans/{}/transactions/{}", plan_id, transaction_id),
                NO_PARAMS,
            )
            .await?;
        Ok((result.data.transaction, result.data.server_knowledge))
    }

    /// Returns all transactions for a specified account, excluding any pending transactions.
    pub fn get_transactions_by_account(
        &self,
        plan_id: PlanId,
        account_id: Uuid,
    ) -> GetTransactionsBuilder<'_> {
        GetTransactionsBuilder {
            client: self,
            scope: TransactionScope::ByAccount(account_id),
            plan_id,
            since_date: None,
            transaction_type: None,
            last_knowledge_of_server: None,
        }
    }

    /// Returns all transactions for a specified category, excluding any pending transactions.
    pub fn get_transactions_by_category(
        &self,
        plan_id: PlanId,
        category_id: Uuid,
    ) -> GetTransactionsBuilder<'_> {
        GetTransactionsBuilder {
            client: self,
            scope: TransactionScope::ByCategory(category_id),
            plan_id,
            since_date: None,
            transaction_type: None,
            last_knowledge_of_server: None,
        }
    }

    /// Returns all transactions for a specified payee, excluding any pending transactions.
    pub fn get_transactions_by_payee(
        &self,
        plan_id: PlanId,
        payee_id: Uuid,
    ) -> GetTransactionsBuilder<'_> {
        GetTransactionsBuilder {
            client: self,
            scope: TransactionScope::ByPayee(payee_id),
            plan_id,
            since_date: None,
            transaction_type: None,
            last_knowledge_of_server: None,
        }
    }

    /// Returns all transactions for a specified month, excluding any pending transactions.
    pub fn get_transactions_by_month(
        &self,
        plan_id: PlanId,
        month: NaiveDate,
    ) -> GetTransactionsBuilder<'_> {
        GetTransactionsBuilder {
            client: self,
            scope: TransactionScope::ByMonth(month),
            plan_id,
            since_date: None,
            transaction_type: None,
            last_knowledge_of_server: None,
        }
    }
}

#[derive(Debug)]
pub struct GetScheduledTransactionsBuilder<'a> {
    client: &'a Client,
    plan_id: PlanId,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetScheduledTransactionsBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> Self {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    pub async fn send(self) -> Result<(Vec<ScheduledTransaction>, i64), Error> {
        let params: Option<&[(&str, &str)]> = if let Some(sk) = self.last_knowledge_of_server {
            Some(&[("last_knowledge_of_server", &sk.to_string())])
        } else {
            None
        };
        let result: ScheduledTransactionsDataEnvelope = self
            .client
            .get(
                &format!("plans/{}/scheduled_transactions", self.plan_id),
                params,
            )
            .await?;
        Ok((
            result.data.scheduled_transactions,
            result.data.server_knowledge,
        ))
    }
}

impl Client {
    /// Returns all scheduled transactions. The second return value is server knowledge for delta
    /// requests.
    pub fn get_scheduled_transactions(
        &self,
        plan_id: PlanId,
    ) -> GetScheduledTransactionsBuilder<'_> {
        GetScheduledTransactionsBuilder {
            client: self,
            plan_id,
            last_knowledge_of_server: None,
        }
    }

    /// Returns a single scheduled transaction.
    pub async fn get_scheduled_transaction(
        &self,
        plan_id: PlanId,
        transaction_id: Uuid,
    ) -> Result<ScheduledTransaction, Error> {
        let result: ScheduledTransactionDataEnvelope = self
            .get(
                &format!(
                    "plans/{}/scheduled_transactions/{}",
                    plan_id, transaction_id
                ),
                NO_PARAMS,
            )
            .await?;
        Ok(result.data.scheduled_transaction)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ImportTransactionsDataEnvelope {
    data: ImportTransactionsData,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImportTransactionsData {
    transaction_ids: Vec<Uuid>,
}

#[derive(Debug, Default, Serialize)]
struct Empty {}

impl Client {
    /// Delete a transaction. Returns deleted transaction and server_knowledge for delta requests
    pub async fn delete_transaction(
        &self,
        plan_id: PlanId,
        tx_id: Uuid,
    ) -> Result<(Transaction, i64), Error> {
        let result: TransactionDataEnvelope = self
            .delete(&format!("plans/{}/transactions/{}", plan_id, tx_id))
            .await?;
        Ok((result.data.transaction, result.data.server_knowledge))
    }

    /// Imports available transactions on all linked accounts for the given
    /// plan. The response for this endpoint contains the transaction
    /// ids that have been imported.
    pub async fn import_transactions(&self, plan_id: PlanId) -> Result<Vec<Uuid>, Error> {
        let result: ImportTransactionsDataEnvelope = self
            .post(
                &format!("plans/{}/transactions/import", plan_id),
                Empty::default(),
            )
            .await?;
        Ok(result.data.transaction_ids)
    }
}

/// A subtransaction within a split transaction to be created or updated.
#[derive(Debug, Serialize)]
pub struct SaveSubTransaction {
    pub amount: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

/// Request body for creating a new transaction.
#[derive(Debug, Serialize)]
pub struct NewTransaction {
    pub account_id: Uuid,
    pub date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared: Option<ClearedStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_color: Option<FlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

/// Request body for updating an existing transaction (PUT single).
#[derive(Debug, Serialize)]
pub struct ExistingTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared: Option<ClearedStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_color: Option<FlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

/// Request body for a single transaction within a batch update (PATCH).
/// Either `id` or `import_id` must be specified to identify the transaction.
#[derive(Debug, Serialize)]
pub struct SaveTransactionWithIdOrImportId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared: Option<ClearedStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_color: Option<FlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

#[derive(Debug, Serialize)]
struct PostTransactionsWrapper {
    transaction: Option<NewTransaction>,
    transactions: Option<Vec<NewTransaction>>,
}

#[derive(Debug, Serialize)]
struct PutTransactionWrapper {
    transaction: ExistingTransaction,
}

#[derive(Debug, Serialize)]
struct PatchTransactionsWrapper {
    transactions: Vec<SaveTransactionWithIdOrImportId>,
}

impl Client {
    /// Creates a single transaction. Returns the full save response including server knowledge.
    pub async fn create_transaction(
        &self,
        plan_id: PlanId,
        transaction: NewTransaction,
    ) -> Result<SaveTransactionsResponse, Error> {
        let result: SaveTransactionsDataEnvelope = self
            .post(
                &format!("plans/{}/transactions", plan_id),
                PostTransactionsWrapper {
                    transaction: Some(transaction),
                    transactions: None,
                },
            )
            .await?;
        Ok(result.data)
    }

    /// Creates multiple transactions. Returns the full save response including server knowledge.
    pub async fn create_transactions(
        &self,
        plan_id: PlanId,
        transactions: Vec<NewTransaction>,
    ) -> Result<SaveTransactionsResponse, Error> {
        let result: SaveTransactionsDataEnvelope = self
            .post(
                &format!("plans/{}/transactions", plan_id),
                PostTransactionsWrapper {
                    transaction: None,
                    transactions: Some(transactions),
                },
            )
            .await?;
        Ok(result.data)
    }

    /// Updates multiple transactions. Returns the full save response including server knowledge.
    pub async fn update_transactions(
        &self,
        plan_id: PlanId,
        transactions: Vec<SaveTransactionWithIdOrImportId>,
    ) -> Result<SaveTransactionsResponse, Error> {
        let result: SaveTransactionsDataEnvelope = self
            .patch(
                &format!("plans/{}/transactions", plan_id),
                PatchTransactionsWrapper { transactions },
            )
            .await?;
        Ok(result.data)
    }

    /// Updates a single transaction. Returns the updated transaction and server knowledge.
    pub async fn update_transaction(
        &self,
        plan_id: PlanId,
        tx_id: Uuid,
        transaction: ExistingTransaction,
    ) -> Result<(Transaction, i64), Error> {
        let result: TransactionDataEnvelope = self
            .put(
                &format!("plans/{}/transactions/{}", plan_id, tx_id),
                PutTransactionWrapper { transaction },
            )
            .await?;
        Ok((result.data.transaction, result.data.server_knowledge))
    }

    /// Creates a scheduled transaction.
    pub async fn create_scheduled_transaction(
        &self,
        plan_id: PlanId,
        scheduled_transaction: SaveScheduledTransaction,
    ) -> Result<ScheduledTransaction, Error> {
        let result: ScheduledTransactionDataEnvelope = self
            .post(
                &format!("plans/{}/scheduled_transactions", plan_id),
                ScheduledTransactionWrapper {
                    scheduled_transaction,
                },
            )
            .await?;
        Ok(result.data.scheduled_transaction)
    }

    /// Updates a scheduled transaction.
    pub async fn update_scheduled_transaction(
        &self,
        plan_id: PlanId,
        scheduled_transaction_id: Uuid,
        scheduled_transaction: SaveScheduledTransaction,
    ) -> Result<ScheduledTransaction, Error> {
        let result: ScheduledTransactionDataEnvelope = self
            .put(
                &format!(
                    "plans/{}/scheduled_transactions/{}",
                    plan_id, scheduled_transaction_id
                ),
                ScheduledTransactionWrapper {
                    scheduled_transaction,
                },
            )
            .await?;
        Ok(result.data.scheduled_transaction)
    }

    /// Deletes a scheduled transaction.
    pub async fn delete_scheduled_transaction(
        &self,
        plan_id: PlanId,
        scheduled_transaction_id: Uuid,
    ) -> Result<ScheduledTransaction, Error> {
        let result: ScheduledTransactionDataEnvelope = self
            .delete(&format!(
                "plans/{}/scheduled_transactions/{}",
                plan_id, scheduled_transaction_id
            ))
            .await?;
        Ok(result.data.scheduled_transaction)
    }
}

/// Request body for creating or updating a scheduled transaction.
#[derive(Debug, Serialize)]
pub struct SaveScheduledTransaction {
    pub account_id: Uuid,
    pub date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_color: Option<FlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<Frequency>,
}

#[derive(Debug, Serialize)]
struct ScheduledTransactionWrapper {
    scheduled_transaction: SaveScheduledTransaction,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{
        TEST_ID_1, TEST_ID_3, TEST_ID_4, error_body, new_test_client,
        scheduled_transaction_fixture, transaction_fixture,
    };
    use serde_json::json;
    use uuid::uuid;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn transactions_list_fixture() -> serde_json::Value {
        json!({ "data": { "transactions": [transaction_fixture()], "server_knowledge": 10 } })
    }

    fn transaction_single_fixture() -> serde_json::Value {
        json!({ "data": { "transaction": transaction_fixture(), "server_knowledge": 10 } })
    }

    fn save_transactions_fixture() -> serde_json::Value {
        json!({
            "data": {
                "transaction_ids": [TEST_ID_1],
                "transaction": transaction_fixture(),
                "transactions": [transaction_fixture()],
                "duplicate_import_ids": null,
                "server_knowledge": 10
            }
        })
    }

    fn scheduled_transactions_list_fixture() -> serde_json::Value {
        json!({
            "data": {
                "scheduled_transactions": [scheduled_transaction_fixture()],
                "server_knowledge": 10
            }
        })
    }

    fn scheduled_transaction_single_fixture() -> serde_json::Value {
        json!({ "data": { "scheduled_transaction": scheduled_transaction_fixture() } })
    }

    fn import_transactions_fixture() -> serde_json::Value {
        json!({ "data": { "transaction_ids": [TEST_ID_1] } })
    }

    #[tokio::test]
    async fn get_transactions_returns_transactions() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/transactions", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(transactions_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (txs, sk) = client
            .get_transactions(PlanId::Id(uuid!(TEST_ID_1)))
            .send()
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].id.to_string(), TEST_ID_1);
        assert_eq!(txs[0].amount, -50000);
        assert_eq!(sk, 10);
    }

    #[tokio::test]
    async fn get_transaction_returns_transaction() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/transactions/{}",
                TEST_ID_1, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(transaction_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (tx, sk) = client
            .get_transaction(PlanId::Id(uuid!(TEST_ID_1)), &uuid!(TEST_ID_1))
            .await
            .unwrap();
        assert_eq!(tx.id.to_string(), TEST_ID_1);
        assert_eq!(tx.amount, -50000);
        assert_eq!(sk, 10);
    }

    #[tokio::test]
    async fn get_transactions_by_account_returns_transactions() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/accounts/{}/transactions",
                TEST_ID_1, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(transactions_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (txs, _) = client
            .get_transactions_by_account(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_1))
            .send()
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
    }

    #[tokio::test]
    async fn get_transactions_by_category_returns_transactions() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/categories/{}/transactions",
                TEST_ID_1, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(transactions_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (txs, _) = client
            .get_transactions_by_category(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_1))
            .send()
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
    }

    #[tokio::test]
    async fn get_transactions_by_payee_returns_transactions() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/payees/{}/transactions",
                TEST_ID_1, TEST_ID_3
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(transactions_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (txs, _) = client
            .get_transactions_by_payee(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_3))
            .send()
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
    }

    #[tokio::test]
    async fn get_transactions_by_month_returns_transactions() {
        let (client, server) = new_test_client().await;
        let month = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/months/{}/transactions",
                TEST_ID_1, month
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(transactions_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (txs, _) = client
            .get_transactions_by_month(PlanId::Id(uuid!(TEST_ID_1)), month)
            .send()
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
    }

    #[tokio::test]
    async fn create_transaction_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/transactions", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(201).set_body_json(save_transactions_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let resp = client
            .create_transaction(
                PlanId::Id(uuid!(TEST_ID_1)),
                NewTransaction {
                    account_id: uuid!(TEST_ID_1),
                    date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                    amount: Some(-50000),
                    memo: None,
                    cleared: Some(ClearedStatus::Cleared),
                    approved: Some(true),
                    payee_id: None,
                    payee_name: None,
                    category_id: None,
                    flag_color: None,
                    import_id: None,
                    subtransactions: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.transaction_ids, vec![uuid!(TEST_ID_1)]);
        assert_eq!(resp.transaction.unwrap().amount, -50000);
    }

    #[tokio::test]
    async fn create_transactions_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/transactions", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(201).set_body_json(save_transactions_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let resp = client
            .create_transactions(
                PlanId::Id(uuid!(TEST_ID_1)),
                vec![NewTransaction {
                    account_id: uuid!(TEST_ID_1),
                    date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                    amount: Some(-50000),
                    memo: None,
                    cleared: Some(ClearedStatus::Cleared),
                    approved: Some(true),
                    payee_id: None,
                    payee_name: None,
                    category_id: None,
                    flag_color: None,
                    import_id: None,
                    subtransactions: None,
                }],
            )
            .await
            .unwrap();
        assert_eq!(resp.transaction_ids, vec![uuid!(TEST_ID_1)]);
    }

    #[tokio::test]
    async fn update_transaction_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("PUT"))
            .and(path(format!(
                "/plans/{}/transactions/{}",
                TEST_ID_1, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(transaction_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (tx, sk) = client
            .update_transaction(
                PlanId::Id(uuid!(TEST_ID_1)),
                uuid!(TEST_ID_1),
                ExistingTransaction {
                    amount: Some(-50000),
                    account_id: None,
                    date: None,
                    payee_id: None,
                    payee_name: None,
                    category_id: None,
                    memo: None,
                    cleared: None,
                    approved: None,
                    flag_color: None,
                    subtransactions: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(tx.id.to_string(), TEST_ID_1);
        assert_eq!(sk, 10);
    }

    #[tokio::test]
    async fn update_transactions_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("PATCH"))
            .and(path(format!("/plans/{}/transactions", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(save_transactions_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let resp = client
            .update_transactions(
                PlanId::Id(uuid!(TEST_ID_1)),
                vec![SaveTransactionWithIdOrImportId {
                    id: Some(Uuid::from_bytes([0; 16])),
                    memo: Some("updated".to_string()),
                    import_id: None,
                    account_id: None,
                    date: None,
                    amount: None,
                    payee_id: None,
                    payee_name: None,
                    category_id: None,
                    cleared: None,
                    approved: None,
                    flag_color: None,
                    subtransactions: None,
                }],
            )
            .await
            .unwrap();
        assert_eq!(resp.transaction_ids, vec![uuid!(TEST_ID_1)]);
    }

    #[tokio::test]
    async fn delete_transaction_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("DELETE"))
            .and(path(format!(
                "/plans/{}/transactions/{}",
                TEST_ID_1, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(transaction_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (tx, sk) = client
            .delete_transaction(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_1))
            .await
            .unwrap();
        assert_eq!(tx.id.to_string(), TEST_ID_1);
        assert_eq!(sk, 10);
    }

    #[tokio::test]
    async fn import_transactions_returns_ids() {
        let (client, server) = new_test_client().await;
        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/transactions/import", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(import_transactions_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let ids = client
            .import_transactions(PlanId::Id(uuid!(TEST_ID_1)))
            .await
            .unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].to_string(), TEST_ID_1);
    }

    #[tokio::test]
    async fn get_scheduled_transactions_returns_transactions() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/scheduled_transactions", TEST_ID_1)))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(scheduled_transactions_list_fixture()),
            )
            .expect(1)
            .mount(&server)
            .await;
        let (txs, sk) = client
            .get_scheduled_transactions(PlanId::Id(uuid!(TEST_ID_1)))
            .send()
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].id.to_string(), TEST_ID_4);
        assert_eq!(sk, 10);
    }

    #[tokio::test]
    async fn get_scheduled_transaction_returns_transaction() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/scheduled_transactions/{}",
                TEST_ID_1, TEST_ID_4
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(scheduled_transaction_single_fixture()),
            )
            .expect(1)
            .mount(&server)
            .await;
        let tx = client
            .get_scheduled_transaction(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_4))
            .await
            .unwrap();
        assert_eq!(tx.id.to_string(), TEST_ID_4);
        assert!(matches!(tx.frequency, Frequency::Monthly));
    }

    #[tokio::test]
    async fn create_scheduled_transaction_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/scheduled_transactions", TEST_ID_1)))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(scheduled_transaction_single_fixture()),
            )
            .expect(1)
            .mount(&server)
            .await;
        let tx = client
            .create_scheduled_transaction(
                PlanId::Id(uuid!(TEST_ID_1)),
                SaveScheduledTransaction {
                    account_id: uuid!(TEST_ID_1),
                    date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    amount: Some(-50000),
                    frequency: Some(Frequency::Monthly),
                    memo: None,
                    payee_id: None,
                    payee_name: None,
                    category_id: None,
                    flag_color: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(tx.id.to_string(), TEST_ID_4);
        assert_eq!(tx.amount, -50000);
    }

    #[tokio::test]
    async fn update_scheduled_transaction_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("PUT"))
            .and(path(format!(
                "/plans/{}/scheduled_transactions/{}",
                TEST_ID_1, TEST_ID_4
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(scheduled_transaction_single_fixture()),
            )
            .expect(1)
            .mount(&server)
            .await;
        let tx = client
            .update_scheduled_transaction(
                PlanId::Id(uuid!(TEST_ID_1)),
                uuid!(TEST_ID_4),
                SaveScheduledTransaction {
                    account_id: uuid!(TEST_ID_1),
                    date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    amount: Some(-50000),
                    frequency: Some(Frequency::Monthly),
                    memo: None,
                    payee_id: None,
                    payee_name: None,
                    category_id: None,
                    flag_color: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(tx.id.to_string(), TEST_ID_4);
    }

    #[tokio::test]
    async fn delete_scheduled_transaction_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("DELETE"))
            .and(path(format!(
                "/plans/{}/scheduled_transactions/{}",
                TEST_ID_1, TEST_ID_4
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(scheduled_transaction_single_fixture()),
            )
            .expect(1)
            .mount(&server)
            .await;
        let tx = client
            .delete_scheduled_transaction(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_4))
            .await
            .unwrap();
        assert_eq!(tx.id.to_string(), TEST_ID_4);
    }

    #[tokio::test]
    async fn get_transaction_returns_not_found() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/transactions/{}",
                TEST_ID_1, TEST_ID_1
            )))
            .respond_with(ResponseTemplate::new(404).set_body_json(error_body(
                "404",
                "not_found",
                "Transaction not found",
            )))
            .mount(&server)
            .await;
        let err = client
            .get_transaction(PlanId::Id(uuid!(TEST_ID_1)), &uuid!(TEST_ID_1))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }
}
