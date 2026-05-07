use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ynab::category::Category;
use crate::ynab::client::Client;
use crate::ynab::errors::Error;

#[derive(Debug, Serialize, Deserialize)]
struct MonthDataEnvelope {
    data: MonthData,
}

#[derive(Debug, Serialize, Deserialize)]
struct MonthData {
    month: Month,
}

#[derive(Debug, Serialize, Deserialize)]
struct MonthsDataEnvelope {
    data: MonthsData,
}

#[derive(Debug, Serialize, Deserialize)]
struct MonthsData {
    months: Vec<Month>,
    server_knowledge: i64,
}

/// Month represents a budget month, including all category allocations and activity.
#[derive(Debug, Serialize, Deserialize)]
pub struct Month {
    pub month: NaiveDate,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<usize>,
    pub deleted: bool,
    pub categories: Vec<Category>,
}

impl Client {
    /// get_months returns all budget months for a plan.
    /// The second return value is server knowledge for delta requests.
    pub async fn get_months(
        &self,
        plan_id: Uuid,
        last_server_knowledge: Option<i64>,
    ) -> Result<Vec<Month>, Error> {
        let sk_str: &str = if let Some(sk) = last_server_knowledge {
            &sk.to_string()
        } else {
            ""
        };
        let mut params: Vec<(&str, &str)> = vec![];
        if !sk_str.is_empty() {
            params.push(("last_knowledge_of_server", sk_str));
        }
        let result: MonthsDataEnvelope = self
            .get(&format!("plans/{}/months", plan_id), &params)
            .await?;
        Ok(result.data.months)
    }

    /// get_month returns a single budget month including its category details.
    pub async fn get_month(&self, plan_id: Uuid, month: NaiveDate) -> Result<Month, Error> {
        let mut params = vec![];
        let result: MonthDataEnvelope = self
            .get(&format!("plans/{}/months/{}", plan_id, month), &params)
            .await?;

        Ok(result.data.month)
    }
}
