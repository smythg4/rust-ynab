use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::PlanId;
use crate::ynab::client::Client;
use crate::ynab::common::NO_PARAMS;
use crate::ynab::errors::Error;

#[derive(Debug, Deserialize)]
struct MoneyMovementsDataEnvelope {
    data: MoneyMovementsData,
}

#[derive(Debug, Deserialize)]
struct MoneyMovementsData {
    money_movements: Vec<MoneyMovement>,
    server_knowledge: i64,
}

#[derive(Debug, Deserialize)]
struct MoneyMovementGroupsDataEnvelope {
    data: MoneyMovementGroupsData,
}

#[derive(Debug, Deserialize)]
struct MoneyMovementGroupsData {
    money_movement_groups: Vec<MoneyMovementGroup>,
    server_knowledge: i64,
}

/// A movement of money between categories. Amounts are in milliunits (divide by 1000 for display).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoneyMovement {
    pub id: Uuid,
    pub month: Option<NaiveDate>,
    pub moved_at: Option<DateTime<chrono::Utc>>,
    pub note: Option<String>,
    pub money_movement_group_id: Option<Uuid>,
    pub performed_by_user_id: Option<Uuid>,
    pub from_category_id: Option<Uuid>,
    pub to_category_id: Option<Uuid>,
    pub amount: i64,
}

/// A group of related money movements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoneyMovementGroup {
    pub id: Uuid,
    pub group_created_at: DateTime<chrono::Utc>,
    pub month: NaiveDate,
    pub note: Option<String>,
    pub performed_by_user_id: Option<Uuid>,
}

#[derive(Debug)]
pub struct GetMoneyMovementsBuilder<'a> {
    client: &'a Client,
    plan_id: PlanId,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetMoneyMovementsBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> Self {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    /// Sends the request. Returns money movements and server knowledge for use in subsequent delta requests.
    pub async fn send(self) -> Result<(Vec<MoneyMovement>, i64), Error> {
        let params: Option<&[(&str, &str)]> = if let Some(sk) = self.last_knowledge_of_server {
            Some(&[("last_knowledge_of_server", &sk.to_string())])
        } else {
            None
        };
        let result: MoneyMovementsDataEnvelope = self
            .client
            .get(&format!("plans/{}/money_movements", self.plan_id), params)
            .await?;
        Ok((result.data.money_movements, result.data.server_knowledge))
    }
}

#[derive(Debug)]
pub struct GetMoneyMovementGroupsBuilder<'a> {
    client: &'a Client,
    plan_id: PlanId,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetMoneyMovementGroupsBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> Self {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    /// Sends the request. Returns money movement groups and server knowledge for use in subsequent delta requests.
    pub async fn send(self) -> Result<(Vec<MoneyMovementGroup>, i64), Error> {
        let params: Option<&[(&str, &str)]> = if let Some(sk) = self.last_knowledge_of_server {
            Some(&[("last_knowledge_of_server", &sk.to_string())])
        } else {
            None
        };
        let result: MoneyMovementGroupsDataEnvelope = self
            .client
            .get(
                &format!("plans/{}/money_movement_groups", self.plan_id),
                params,
            )
            .await?;
        Ok((
            result.data.money_movement_groups,
            result.data.server_knowledge,
        ))
    }
}

impl Client {
    /// Returns a builder for fetching all money movements. Chain `.with_server_knowledge()` for a delta request.
    pub fn get_money_movements(&self, plan_id: PlanId) -> GetMoneyMovementsBuilder<'_> {
        GetMoneyMovementsBuilder {
            client: self,
            plan_id,
            last_knowledge_of_server: None,
        }
    }

    /// Returns all money movements for a specific month. The second return value is server
    /// knowledge for delta requests.
    pub async fn get_money_movements_by_month(
        &self,
        plan_id: PlanId,
        month: NaiveDate,
    ) -> Result<(Vec<MoneyMovement>, i64), Error> {
        let result: MoneyMovementsDataEnvelope = self
            .get(
                &format!("plans/{}/months/{}/money_movements", plan_id, month),
                NO_PARAMS,
            )
            .await?;
        Ok((result.data.money_movements, result.data.server_knowledge))
    }

    /// Returns a builder for fetching all money movement groups. Chain `.with_server_knowledge()` for a delta request.
    pub fn get_money_movement_groups(&self, plan_id: PlanId) -> GetMoneyMovementGroupsBuilder<'_> {
        GetMoneyMovementGroupsBuilder {
            client: self,
            plan_id,
            last_knowledge_of_server: None,
        }
    }

    /// Returns all money movement groups for a specific month. The second return value is server
    /// knowledge for delta requests.
    pub async fn get_money_movement_groups_by_month(
        &self,
        plan_id: PlanId,
        month: NaiveDate,
    ) -> Result<(Vec<MoneyMovementGroup>, i64), Error> {
        let result: MoneyMovementGroupsDataEnvelope = self
            .get(
                &format!("plans/{}/months/{}/money_movement_groups", plan_id, month),
                NO_PARAMS,
            )
            .await?;
        Ok((
            result.data.money_movement_groups,
            result.data.server_knowledge,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{TEST_ID_1, TEST_ID_2, new_test_client};
    use serde_json::json;
    use uuid::uuid;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn movement_fixture() -> serde_json::Value {
        json!({
            "id": TEST_ID_1,
            "month": "2024-01-01",
            "moved_at": null,
            "note": null,
            "money_movement_group_id": null,
            "performed_by_user_id": null,
            "from_category_id": TEST_ID_2,
            "to_category_id": TEST_ID_1,
            "amount": 10000
        })
    }

    fn movement_group_fixture() -> serde_json::Value {
        json!({
            "id": TEST_ID_1,
            "group_created_at": "2024-01-01T00:00:00Z",
            "month": "2024-01-01",
            "note": null,
            "performed_by_user_id": null
        })
    }

    fn movements_list_fixture() -> serde_json::Value {
        json!({ "data": { "money_movements": [movement_fixture()], "server_knowledge": 4 } })
    }

    fn movement_groups_list_fixture() -> serde_json::Value {
        json!({ "data": { "money_movement_groups": [movement_group_fixture()], "server_knowledge": 4 } })
    }

    #[tokio::test]
    async fn get_money_movements_returns_movements() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/money_movements", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(movements_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (movements, sk) = client
            .get_money_movements(PlanId::Id(uuid!(TEST_ID_1)))
            .send()
            .await
            .unwrap();
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].id.to_string(), TEST_ID_1);
        assert_eq!(movements[0].amount, 10000);
        assert_eq!(sk, 4);
    }

    #[tokio::test]
    async fn get_money_movements_by_month_returns_movements() {
        let (client, server) = new_test_client().await;
        let month = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/months/{}/money_movements",
                TEST_ID_1, month
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(movements_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (movements, sk) = client
            .get_money_movements_by_month(PlanId::Id(uuid!(TEST_ID_1)), month)
            .await
            .unwrap();
        assert_eq!(movements.len(), 1);
        assert_eq!(sk, 4);
    }

    #[tokio::test]
    async fn get_money_movement_groups_returns_groups() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/money_movement_groups", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(movement_groups_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (groups, sk) = client
            .get_money_movement_groups(PlanId::Id(uuid!(TEST_ID_1)))
            .send()
            .await
            .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].id.to_string(), TEST_ID_1);
        assert_eq!(sk, 4);
    }

    #[tokio::test]
    async fn get_money_movement_groups_by_month_returns_groups() {
        let (client, server) = new_test_client().await;
        let month = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/months/{}/money_movement_groups",
                TEST_ID_1, month
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(movement_groups_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (groups, sk) = client
            .get_money_movement_groups_by_month(PlanId::Id(uuid!(TEST_ID_1)), month)
            .await
            .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(sk, 4);
    }
}
