use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ynab::client::Client;
use crate::ynab::errors::Error;
use crate::PlanId;
use crate::ynab::common::NO_PARAMS;

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

/// MoneyMovement represents a single movement of money between categories.
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneyMovement {
    pub id: Uuid,
    pub month: NaiveDate,
    pub moved_at: Option<DateTime<chrono::Utc>>,
    pub note: Option<String>,
    pub money_movement_group_id: Option<Uuid>,
    pub performed_by_user_id: Option<Uuid>,
    pub from_category_id: Option<Uuid>,
    pub to_category_id: Option<Uuid>,
    pub amount: i64,
}

/// MoneyMovementGroup represents a group of related money movements.
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneyMovementGroup {
    pub id: Uuid,
    pub group_created_at: DateTime<chrono::Utc>,
    pub month: NaiveDate,
    pub note: Option<String>,
    pub performed_by_user_id: Option<Uuid>,
}

impl Client {
    /// get_money_movements returns all money movements for a plan.
    /// The second return value is server knowledge for delta requests.
    pub async fn get_money_movements(
        &self,
        plan_id: PlanId,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<(Vec<MoneyMovement>, i64), Error> {
        let sk_owned = last_knowledge_of_server.map(|sk| sk.to_string());
        let params: Vec<(&str, &str)> = sk_owned
            .as_deref()
            .map(|sk| vec![("last_knowledge_of_server", sk)])
            .unwrap_or_default();
        let result: MoneyMovementsDataEnvelope = self
            .get(&format!("plans/{}/money_movements", plan_id), &params)
            .await?;
        Ok((result.data.money_movements, result.data.server_knowledge))
    }

    /// get_money_movements_by_month returns money movements for a specific budget month.
    /// The second return value is server knowledge for delta requests.
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

    /// get_money_movement_groups returns all money movement groups for a plan.
    /// The second return value is server knowledge for delta requests.
    pub async fn get_money_movement_groups(
        &self,
        plan_id: PlanId,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<(Vec<MoneyMovementGroup>, i64), Error> {
        let sk_owned = last_knowledge_of_server.map(|sk| sk.to_string());
        let params: Vec<(&str, &str)> = sk_owned
            .as_deref()
            .map(|sk| vec![("last_knowledge_of_server", sk)])
            .unwrap_or_default();
        let result: MoneyMovementGroupsDataEnvelope = self
            .get(&format!("plans/{}/money_movement_groups", plan_id), &params)
            .await?;
        Ok((
            result.data.money_movement_groups,
            result.data.server_knowledge,
        ))
    }

    /// get_money_movement_groups_by_month returns money movement groups for a specific budget month.
    /// The second return value is server knowledge for delta requests.
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
