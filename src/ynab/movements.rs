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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
    /// Returns all money movements. The second return value is server knowledge for delta requests.
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

    /// Returns all money movement groups. The second return value is server knowledge for delta
    /// requests.
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
