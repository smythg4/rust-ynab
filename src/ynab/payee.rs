use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::PlanId;
use crate::ynab::client::Client;
use crate::ynab::common::NO_PARAMS;
use crate::ynab::errors::Error;

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

/// A payee for a plan.
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

/// A GPS location stored when a transaction is entered on a mobile device. Locations will not be
/// available for all payees.
#[derive(Debug, Deserialize, Serialize)]
pub struct PayeeLocation {
    pub id: Uuid,
    pub payee_id: Uuid,
    pub latitude: String,
    pub longitude: String,
    pub deleted: bool,
}

#[derive(Debug)]
pub struct GetPayeesBuilder<'a> {
    client: &'a Client,
    plan_id: PlanId,
    last_knowledge_of_server: Option<i64>,
}

impl<'a> GetPayeesBuilder<'a> {
    pub fn with_server_knowledge(mut self, sk: i64) -> Self {
        self.last_knowledge_of_server = Some(sk);
        self
    }

    pub async fn send(self) -> Result<(Vec<Payee>, i64), Error> {
        let result: PayeesDataEnvelope = self
            .client
            .get(&format!("plans/{}/payees", self.plan_id), NO_PARAMS)
            .await?;
        Ok((result.data.payees, result.data.server_knowledge))
    }
}

impl Client {
    /// Returns all payees. The second return value is server knowledge for delta requests.
    pub fn get_payees(&self, plan_id: PlanId) -> GetPayeesBuilder<'_> {
        GetPayeesBuilder {
            client: self,
            plan_id,
            last_knowledge_of_server: None,
        }
    }
    /// Returns a single payee.
    pub async fn get_payee(&self, plan_id: PlanId, payee_id: Uuid) -> Result<Payee, Error> {
        let result: PayeeDataEnvelope = self
            .get(&format!("plans/{}/payees/{}", plan_id, payee_id), NO_PARAMS)
            .await?;
        Ok(result.data.payee)
    }

    /// Returns all payee locations.
    pub async fn get_payee_locations(&self, plan_id: PlanId) -> Result<Vec<PayeeLocation>, Error> {
        let result: PayeeLocationsDataEnvelope = self
            .get(&format!("plans/{}/payee_locations", plan_id), NO_PARAMS)
            .await?;
        Ok(result.data.payee_locations)
    }

    /// Returns all payee locations for a specified payee.
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

    /// Returns a single payee location.
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
