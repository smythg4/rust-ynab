use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::ynab::category::Category;
use crate::ynab::client::Client;
use crate::ynab::errors::Error;
use crate::PlanId;
use crate::ynab::common::NO_PARAMS;

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
    pub categories: Option<Vec<Category>>,
}

impl Client {
    /// get_months returns all budget months for a plan.
    /// The second return value is server knowledge for delta requests.
    pub async fn get_months(
        &self,
        plan_id: PlanId,
        last_server_knowledge: Option<i64>,
    ) -> Result<Vec<Month>, Error> {
        let sk_owned = last_server_knowledge.map(|sk| sk.to_string());
        let params: Vec<(&str, &str)> = sk_owned
            .as_deref()
            .map(|sk| vec![("last_knowledge_of_server", sk)])
            .unwrap_or_default();

        let result: MonthsDataEnvelope = self
            .get(&format!("plans/{}/months", plan_id), &params)
            .await?;
        Ok(result.data.months)
    }

    /// get_month returns a single budget month including its category details.
    pub async fn get_month(&self, plan_id: PlanId, month: NaiveDate) -> Result<Month, Error> {
        let result: MonthDataEnvelope = self
            .get(&format!("plans/{}/months/{}", plan_id, month), NO_PARAMS)
            .await?;

        Ok(result.data.month)
    }
}
