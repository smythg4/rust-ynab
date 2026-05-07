use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ynab::client::Client;
use crate::ynab::errors::Error;
use crate::PlanId;
use crate::ynab::common::NO_PARAMS;

#[derive(Debug, Deserialize, Serialize)]
struct PayeesDataEnvelope {
    data: PayeesData,
}

#[derive(Debug, Deserialize, Serialize)]
struct PayeesData {
    payees: Vec<Payee>,
    server_knowledge: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct PayeeDataEnvelope {
    data: PayeeData,
}

#[derive(Debug, Deserialize, Serialize)]
struct PayeeData {
    payee: Payee,
}

/// Payee represents a payee that can be associated with transactions.
#[derive(Debug, Deserialize, Serialize)]
pub struct Payee {
    pub id: Uuid,
    pub name: String,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct PayeeLocationDataEnvelope {
    data: PayeeLocationData,
}

#[derive(Debug, Deserialize, Serialize)]
struct PayeeLocationData {
    payee_location: PayeeLocation,
}

#[derive(Debug, Deserialize, Serialize)]
struct PayeeLocationsDataEnvelope {
    data: PayeeLocationsData,
}

#[derive(Debug, Deserialize, Serialize)]
struct PayeeLocationsData {
    payee_locations: Vec<PayeeLocation>,
}

/// PayeeLocation represents a geographic location associated with a payee.
#[derive(Debug, Deserialize, Serialize)]
pub struct PayeeLocation {
    pub id: Uuid,
    pub payee_id: Uuid,
    pub latitude: String,
    pub longitude: String,
    pub deleted: bool,
}

impl Client {
    /// get_payees returns all payees for a plan. The second return value is server knowledge for delta requests.
    pub async fn get_payees(&self, plan_id: PlanId) -> Result<Vec<Payee>, Error> {
        let result: PayeesDataEnvelope = self
            .get(&format!("plans/{}/payees", plan_id), NO_PARAMS)
            .await?;
        Ok(result.data.payees)
    }
    /// get_payee returns a single payee by ID.
    pub async fn get_payee(&self, plan_id: PlanId, payee_id: Uuid) -> Result<Payee, Error> {
        let result: PayeeDataEnvelope = self
            .get(&format!("plans/{}/payees/{}", plan_id, payee_id), NO_PARAMS)
            .await?;
        Ok(result.data.payee)
    }

    /// get_payee_locations returns all locations associated with a plan.
    pub async fn get_payee_locations(&self, plan_id: PlanId) -> Result<Vec<PayeeLocation>, Error> {
        let result: PayeeLocationsDataEnvelope = self
            .get(&format!("plans/{}/payee_locations", plan_id), NO_PARAMS)
            .await?;
        Ok(result.data.payee_locations)
    }

    /// get_payee_locations_by_payee returns all locations associated with a specific payee.
    pub async fn get_payee_locations_by_payee(
        &self,
        plan_id: PlanId,
        payee_id: Uuid,
    ) -> Result<Vec<PayeeLocation>, Error> {
        let result: PayeeLocationsDataEnvelope = self
            .get(
                &format!("plans/{}/payees/{}/payee_locations", plan_id, payee_id),
                NO_PARAMS,
            )
            .await?;
        Ok(result.data.payee_locations)
    }

    /// get_payee_location returns the location by ID
    pub async fn get_payee_location(
        &self,
        plan_id: PlanId,
        location_id: Uuid,
    ) -> Result<PayeeLocation, Error> {
        let result: PayeeLocationDataEnvelope = self
            .get(
                &format!("plans/{}/payee_locations/{}", plan_id, location_id),
                NO_PARAMS,
            )
            .await?;
        Ok(result.data.payee_location)
    }
}
