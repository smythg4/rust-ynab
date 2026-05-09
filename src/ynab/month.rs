use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::PlanId;
use crate::ynab::category::Category;
use crate::ynab::client::Client;
use crate::ynab::common::NO_PARAMS;
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

/// A plan month. This is where Ready to Assign, Age of Money, and category amounts
/// (assigned, activity, available) are available. Amounts are in milliunits (divide by 1000 for
/// display).
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
    #[serde(default)]
    pub categories: Vec<Category>,
}

#[derive(Debug)]
pub struct GetMonthsBuilder<'a> {
    client: &'a Client,
    plan_id: PlanId,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetMonthsBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> Self {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    pub async fn send(self) -> Result<(Vec<Month>, i64), Error> {
        let params: Option<&[(&str, &str)]> = if let Some(sk) = self.last_knowledge_of_server {
            Some(&[("last_knowledge_of_server", &sk.to_string())])
        } else {
            None
        };
        let result: MonthsDataEnvelope = self
            .client
            .get(&format!("plans/{}/months", self.plan_id), params)
            .await?;
        Ok((result.data.months, result.data.server_knowledge))
    }
}

impl Client {
    /// Returns all plan months. The second return value is server knowledge for delta requests.
    pub fn get_months(&self, plan_id: PlanId) -> GetMonthsBuilder<'_> {
        GetMonthsBuilder {
            client: self,
            plan_id,
            last_knowledge_of_server: None,
        }
    }

    /// Returns a single plan month.
    pub async fn get_month(&self, plan_id: PlanId, month: NaiveDate) -> Result<Month, Error> {
        let result: MonthDataEnvelope = self
            .get(&format!("plans/{}/months/{}", plan_id, month), NO_PARAMS)
            .await?;
        Ok(result.data.month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{TEST_ID_1, month_fixture, new_test_client};
    use serde_json::json;
    use uuid::uuid;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn months_list_fixture() -> serde_json::Value {
        json!({ "data": { "months": [month_fixture()], "server_knowledge": 6 } })
    }

    fn month_single_fixture() -> serde_json::Value {
        json!({ "data": { "month": month_fixture() } })
    }

    #[tokio::test]
    async fn get_months_returns_months() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/months", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(months_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (months, sk) = client
            .get_months(PlanId::Id(uuid!(TEST_ID_1)))
            .send()
            .await
            .unwrap();
        assert_eq!(months.len(), 1);
        assert_eq!(months[0].income, 500000);
        assert_eq!(sk, 6);
    }

    #[tokio::test]
    async fn get_month_returns_month() {
        let (client, server) = new_test_client().await;
        let month = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/months/{}", TEST_ID_1, month)))
            .respond_with(ResponseTemplate::new(200).set_body_json(month_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let m = client
            .get_month(PlanId::Id(uuid!(TEST_ID_1)), month)
            .await
            .unwrap();
        assert_eq!(m.income, 500000);
        assert_eq!(m.categories.len(), 1);
    }

    #[test]
    fn deserializes_without_optional_fields() {
        let json = r#"{ "month": "2024-01-01", "note": null, "income": 0,
              "budgeted": 0, "activity": 0, "to_be_budgeted": 0,
              "age_of_money": null, "deleted": false }"#;
        let month: Month = serde_json::from_str(json).unwrap();
        assert!(month.categories.is_empty());
    }
}
