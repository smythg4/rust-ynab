use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ynab::client::Client;
use crate::ynab::errors::Error;
use crate::PlanId;
use crate::ynab::common::NO_PARAMS;

// --- Envelopes ---

#[derive(Debug, Deserialize)]
struct TransactionDataEnvelope {
    data: TransactionData,
}

#[derive(Debug, Deserialize)]
struct TransactionData {
    transaction: Transaction,
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
struct CreateTransactionResponseEnvelope {
    data: CreateTransactionResponse,
}

#[derive(Debug, Deserialize)]
struct CreateTransactionsResponseEnvelope {
    data: CreateTransactionsResponse,
}

#[derive(Debug, Deserialize)]
struct ImportTransactionsResponseEnvelope {
    data: ImportTransactionsResponse,
}

// --- Enums ---

/// ClearedStatus represents the cleared state of a transaction.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClearedStatus {
    Cleared,
    Uncleared,
    Reconciled,
}

/// FlagColor represents the color of a transaction flag.
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

/// Frequency represents the recurrence interval for a scheduled transaction.
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
    #[serde(rename = "everyThreeMonths")]
    EveryThreeMonths,
    #[serde(rename = "everyFourMonths")]
    EveryFourMonths,
    #[serde(rename = "twiceAYear")]
    TwiceAYear,
    #[serde(rename = "yearly")]
    Yearly,
    #[serde(rename = "everyOtherYear")]
    EveryOtherYear,
}

// --- Read types ---

/// Transaction represents a single YNAB transaction. Amounts are in milliunits (divide by 1000 for display).
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub date: NaiveDate,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedStatus,
    pub approved: bool,
    pub flag_color: Option<FlagColor>,
    pub flag_name: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub matched_transaction_id: Option<String>,
    pub subtransactions: Vec<Subtransaction>,
}

/// Subtransaction is a line item within a split transaction.
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

/// ScheduledTransaction represents a recurring scheduled transaction.
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

/// ScheduledSubtransaction is a line item within a split scheduled transaction.
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

// --- Write types ---

/// SaveTransaction is the request body for creating a transaction. Amount is in milliunits.
#[derive(Debug, Serialize)]
pub struct SaveTransaction {
    pub account_id: Uuid,
    pub date: NaiveDate,
    pub amount: i64,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subtransactions: Vec<SaveSubtransaction>,
}

/// SaveSubtransaction is the request body for a sub-transaction. Amount is in milliunits.
#[derive(Debug, Serialize)]
pub struct SaveSubtransaction {
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

/// UpdateTransaction is the request body for updating a transaction.
#[derive(Debug, Serialize)]
pub struct UpdateTransaction {
    pub id: String,
    pub account_id: Uuid,
    pub date: NaiveDate,
    pub amount: i64,
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
}

/// SaveScheduledTransaction is the request body for creating or updating a scheduled transaction.
#[derive(Debug, Serialize)]
pub struct SaveScheduledTransaction {
    pub account_id: Uuid,
    pub date: NaiveDate,
    pub amount: i64,
    pub frequency: Frequency,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_color: Option<FlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_name: Option<String>,
}

// --- Response types ---

/// CreateTransactionResponse is returned by create_transaction.
#[derive(Debug, Deserialize)]
pub struct CreateTransactionResponse {
    pub transaction_ids: Vec<String>,
    pub transaction: Transaction,
    pub duplicate_import_ids: Vec<String>,
    pub server_knowledge: i64,
}

/// CreateTransactionsResponse is returned by create_transactions and update_transactions.
#[derive(Debug, Deserialize)]
pub struct CreateTransactionsResponse {
    pub transaction_ids: Vec<String>,
    pub transactions: Vec<Transaction>,
    pub duplicate_import_ids: Vec<String>,
    pub server_knowledge: i64,
}

/// ImportTransactionsResponse is returned by import_transactions.
#[derive(Debug, Deserialize)]
pub struct ImportTransactionsResponse {
    pub transaction_ids: Vec<String>,
    pub server_knowledge: i64,
}

// --- Params ---

/// TransactionListParams holds optional filter parameters for transaction list endpoints.
pub struct TransactionListParams {
    /// Only return transactions on or after this date.
    pub since_date: Option<NaiveDate>,
    /// Filter by "uncategorized" or "unapproved".
    pub transaction_type: Option<String>,
    /// For delta requests; pass the value returned by a prior call.
    pub last_knowledge_of_server: Option<i64>,
}

fn build_transaction_params(params: Option<&TransactionListParams>) -> Vec<(String, String)> {
    let mut q = vec![];
    if let Some(p) = params {
        if let Some(since_date) = &p.since_date {
            q.push(("since_date".to_string(), since_date.to_string()));
        }
        if let Some(tx_type) = &p.transaction_type {
            q.push(("type".to_string(), tx_type.clone()));
        }
        if let Some(sk) = p.last_knowledge_of_server {
            q.push(("last_knowledge_of_server".to_string(), sk.to_string()));
        }
    }
    q
}

// --- Request wrappers ---

#[derive(Serialize)]
struct SaveTransactionWrapper {
    transaction: SaveTransaction,
}

#[derive(Serialize)]
struct SaveTransactionsWrapper {
    transactions: Vec<SaveTransaction>,
}

#[derive(Serialize)]
struct UpdateTransactionWrapper {
    transaction: UpdateTransaction,
}

#[derive(Serialize)]
struct UpdateTransactionsWrapper {
    transactions: Vec<UpdateTransaction>,
}

#[derive(Serialize)]
struct SaveScheduledTransactionWrapper {
    scheduled_transaction: SaveScheduledTransaction,
}

#[derive(Serialize)]
struct EmptyBody {}

// --- impl Client ---

impl Client {
    /// get_transactions returns all transactions for a plan.
    /// The second return value is server knowledge for delta requests.
    pub async fn get_transactions(
        &self,
        plan_id: PlanId,
        params: Option<&TransactionListParams>,
    ) -> Result<(Vec<Transaction>, i64), Error> {
        let owned = build_transaction_params(params);
        let refs: Vec<(&str, &str)> = owned
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let result: TransactionsDataEnvelope = self
            .get(&format!("plans/{}/transactions", plan_id), &refs)
            .await?;
        Ok((result.data.transactions, result.data.server_knowledge))
    }

    /// get_transaction returns a single transaction by ID.
    pub async fn get_transaction(
        &self,
        plan_id: PlanId,
        transaction_id: &str,
    ) -> Result<Transaction, Error> {
        let result: TransactionDataEnvelope = self
            .get(
                &format!("plans/{}/transactions/{}", plan_id, transaction_id),
                NO_PARAMS,
            )
            .await?;
        Ok(result.data.transaction)
    }

    /// get_transactions_by_account returns all transactions for a specific account.
    pub async fn get_transactions_by_account(
        &self,
        plan_id: PlanId,
        account_id: Uuid,
        params: Option<&TransactionListParams>,
    ) -> Result<(Vec<Transaction>, i64), Error> {
        let owned = build_transaction_params(params);
        let refs: Vec<(&str, &str)> = owned
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let result: TransactionsDataEnvelope = self
            .get(
                &format!("plans/{}/accounts/{}/transactions", plan_id, account_id),
                &refs,
            )
            .await?;
        Ok((result.data.transactions, result.data.server_knowledge))
    }

    /// get_transactions_by_category returns all transactions for a specific category.
    pub async fn get_transactions_by_category(
        &self,
        plan_id: PlanId,
        category_id: Uuid,
        params: Option<&TransactionListParams>,
    ) -> Result<(Vec<Transaction>, i64), Error> {
        let owned = build_transaction_params(params);
        let refs: Vec<(&str, &str)> = owned
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let result: TransactionsDataEnvelope = self
            .get(
                &format!("plans/{}/categories/{}/transactions", plan_id, category_id),
                &refs,
            )
            .await?;
        Ok((result.data.transactions, result.data.server_knowledge))
    }

    /// get_transactions_by_payee returns all transactions for a specific payee.
    pub async fn get_transactions_by_payee(
        &self,
        plan_id: PlanId,
        payee_id: Uuid,
        params: Option<&TransactionListParams>,
    ) -> Result<(Vec<Transaction>, i64), Error> {
        let owned = build_transaction_params(params);
        let refs: Vec<(&str, &str)> = owned
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let result: TransactionsDataEnvelope = self
            .get(
                &format!("plans/{}/payees/{}/transactions", plan_id, payee_id),
                &refs,
            )
            .await?;
        Ok((result.data.transactions, result.data.server_knowledge))
    }

    /// get_transactions_by_month returns all transactions for a specific budget month.
    pub async fn get_transactions_by_month(
        &self,
        plan_id: PlanId,
        month: NaiveDate,
        params: Option<&TransactionListParams>,
    ) -> Result<(Vec<Transaction>, i64), Error> {
        let owned = build_transaction_params(params);
        let refs: Vec<(&str, &str)> = owned
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let result: TransactionsDataEnvelope = self
            .get(
                &format!("plans/{}/months/{}/transactions", plan_id, month),
                &refs,
            )
            .await?;
        Ok((result.data.transactions, result.data.server_knowledge))
    }

    /// get_scheduled_transactions returns all scheduled transactions for a plan.
    /// The second return value is server knowledge for delta requests.
    pub async fn get_scheduled_transactions(
        &self,
        plan_id: PlanId,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<(Vec<ScheduledTransaction>, i64), Error> {
        let sk_owned = last_knowledge_of_server.map(|sk| sk.to_string());
        let params: Vec<(&str, &str)> = sk_owned
            .as_deref()
            .map(|sk| vec![("last_knowledge_of_server", sk)])
            .unwrap_or_default();
        let result: ScheduledTransactionsDataEnvelope = self
            .get(
                &format!("plans/{}/scheduled_transactions", plan_id),
                &params,
            )
            .await?;
        Ok((
            result.data.scheduled_transactions,
            result.data.server_knowledge,
        ))
    }

    /// get_scheduled_transaction returns a single scheduled transaction by ID.
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

    /// create_transaction creates a single transaction.
    pub async fn create_transaction(
        &self,
        plan_id: PlanId,
        transaction: SaveTransaction,
    ) -> Result<CreateTransactionResponse, Error> {
        let result: CreateTransactionResponseEnvelope = self
            .post(
                &format!("plans/{}/transactions", plan_id),
                &SaveTransactionWrapper { transaction },
            )
            .await?;
        Ok(result.data)
    }

    /// create_transactions creates multiple transactions in a single request.
    pub async fn create_transactions(
        &self,
        plan_id: PlanId,
        transactions: Vec<SaveTransaction>,
    ) -> Result<CreateTransactionsResponse, Error> {
        let result: CreateTransactionsResponseEnvelope = self
            .post(
                &format!("plans/{}/transactions", plan_id),
                &SaveTransactionsWrapper { transactions },
            )
            .await?;
        Ok(result.data)
    }

    /// import_transactions triggers an import from linked accounts.
    pub async fn import_transactions(
        &self,
        plan_id: PlanId,
    ) -> Result<ImportTransactionsResponse, Error> {
        let result: ImportTransactionsResponseEnvelope = self
            .post(
                &format!("plans/{}/transactions/import", plan_id),
                &EmptyBody {},
            )
            .await?;
        Ok(result.data)
    }

    /// create_scheduled_transaction creates a new scheduled transaction.
    pub async fn create_scheduled_transaction(
        &self,
        plan_id: PlanId,
        transaction: SaveScheduledTransaction,
    ) -> Result<ScheduledTransaction, Error> {
        let result: ScheduledTransactionDataEnvelope = self
            .post(
                &format!("plans/{}/scheduled_transactions", plan_id),
                &SaveScheduledTransactionWrapper { transaction },
            )
            .await?;
        Ok(result.data.scheduled_transaction)
    }

    /// update_transaction replaces a transaction.
    pub async fn update_transaction(
        &self,
        plan_id: PlanId,
        transaction_id: &str,
        transaction: UpdateTransaction,
    ) -> Result<CreateTransactionResponse, Error> {
        let result: CreateTransactionResponseEnvelope = self
            .put(
                &format!("plans/{}/transactions/{}", plan_id, transaction_id),
                &UpdateTransactionWrapper { transaction },
            )
            .await?;
        Ok(result.data)
    }

    /// update_scheduled_transaction replaces a scheduled transaction.
    pub async fn update_scheduled_transaction(
        &self,
        plan_id: PlanId,
        transaction_id: Uuid,
        transaction: SaveScheduledTransaction,
    ) -> Result<ScheduledTransaction, Error> {
        let result: ScheduledTransactionDataEnvelope = self
            .put(
                &format!(
                    "plans/{}/scheduled_transactions/{}",
                    plan_id, transaction_id
                ),
                &SaveScheduledTransactionWrapper { transaction },
            )
            .await?;
        Ok(result.data.scheduled_transaction)
    }

    /// update_transactions applies partial updates to multiple transactions.
    pub async fn update_transactions(
        &self,
        plan_id: PlanId,
        transactions: Vec<UpdateTransaction>,
    ) -> Result<CreateTransactionsResponse, Error> {
        let result: CreateTransactionsResponseEnvelope = self
            .patch(
                &format!("plans/{}/transactions", plan_id),
                &UpdateTransactionsWrapper { transactions },
            )
            .await?;
        Ok(result.data)
    }

    /// delete_transaction deletes a transaction and returns the deleted record.
    pub async fn delete_transaction(
        &self,
        plan_id: PlanId,
        transaction_id: &str,
    ) -> Result<Transaction, Error> {
        let result: TransactionDataEnvelope = self
            .delete(&format!(
                "plans/{}/transactions/{}",
                plan_id, transaction_id
            ))
            .await?;
        Ok(result.data.transaction)
    }

    /// delete_scheduled_transaction deletes a scheduled transaction and returns the deleted record.
    pub async fn delete_scheduled_transaction(
        &self,
        plan_id: PlanId,
        transaction_id: Uuid,
    ) -> Result<ScheduledTransaction, Error> {
        let result: ScheduledTransactionDataEnvelope = self
            .delete(&format!(
                "plans/{}/scheduled_transactions/{}",
                plan_id, transaction_id
            ))
            .await?;
        Ok(result.data.scheduled_transaction)
    }
}
