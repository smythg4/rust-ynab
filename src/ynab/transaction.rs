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
    pub account_name: Option<String>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub matched_transaction_id: Option<String>,
    #[serde(default)]
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
