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
    server_knowledge: i64,
}

/// A payee for a plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

    /// Sends the request. Returns payees and server knowledge for use in subsequent delta requests.
    pub async fn send(self) -> Result<(Vec<Payee>, i64), Error> {
        let result: PayeesDataEnvelope = self
            .client
            .get(&format!("plans/{}/payees", self.plan_id), NO_PARAMS)
            .await?;
        Ok((result.data.payees, result.data.server_knowledge))
    }
}

impl Client {
    /// Returns a builder for fetching all payees. Chain `.with_server_knowledge()` for a delta request.
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

/// Request body for creating a new payee. Name is required and must not exceed 500
/// characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostPayee {
    pub name: String,
}
#[derive(Debug, Serialize)]
struct PostPayeeWrapper {
    payee: PostPayee,
}

/// Request body for updating an existing payee. All fields are optional; omitted fields are
/// not changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SavePayee {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
#[derive(Debug, Serialize)]
struct PatchPayeeWrapper {
    payee: SavePayee,
}

impl Client {
    /// Creates a new payee. Returns the created payee and server knowledge for delta requests.
    pub async fn create_payee(
        &self,
        plan_id: PlanId,
        payee: PostPayee,
    ) -> Result<(Payee, i64), Error> {
        let result: PayeeDataEnvelope = self
            .post(
                &format!("plans/{}/payees", plan_id),
                PostPayeeWrapper { payee },
            )
            .await?;
        Ok((result.data.payee, result.data.server_knowledge))
    }

    /// Updates an existing payee. Returns the updated payee and server knowledge for delta
    /// requests.
    pub async fn update_payee(
        &self,
        plan_id: PlanId,
        payee_id: Uuid,
        payee: SavePayee,
    ) -> Result<(Payee, i64), Error> {
        let result: PayeeDataEnvelope = self
            .patch(
                &format!("plans/{}/payees/{}", plan_id, payee_id),
                PatchPayeeWrapper { payee },
            )
            .await?;
        Ok((result.data.payee, result.data.server_knowledge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{
        TEST_ID_1, TEST_ID_3, TEST_ID_4, error_body, new_test_client, payee_fixture,
        payee_location_fixture,
    };
    use serde_json::json;
    use uuid::uuid;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn payees_list_fixture() -> serde_json::Value {
        json!({ "data": { "payees": [payee_fixture()], "server_knowledge": 3 } })
    }

    fn payee_single_fixture() -> serde_json::Value {
        json!({ "data": { "payee": payee_fixture(), "server_knowledge": 3 } })
    }

    fn payee_locations_list_fixture() -> serde_json::Value {
        json!({ "data": { "payee_locations": [payee_location_fixture()] } })
    }

    fn payee_location_single_fixture() -> serde_json::Value {
        json!({ "data": { "payee_location": payee_location_fixture() } })
    }

    #[tokio::test]
    async fn get_payees_returns_payees() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/payees", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(payees_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (payees, sk) = client
            .get_payees(PlanId::Id(uuid!(TEST_ID_1)))
            .send()
            .await
            .unwrap();
        assert_eq!(payees.len(), 1);
        assert_eq!(payees[0].id.to_string(), TEST_ID_3);
        assert_eq!(sk, 3);
    }

    #[tokio::test]
    async fn get_payee_returns_payee() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/payees/{}", TEST_ID_1, TEST_ID_3)))
            .respond_with(ResponseTemplate::new(200).set_body_json(payee_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let payee = client
            .get_payee(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_3))
            .await
            .unwrap();
        assert_eq!(payee.id.to_string(), TEST_ID_3);
        assert_eq!(payee.name, "Amazon");
    }

    #[tokio::test]
    async fn get_payee_locations_returns_locations() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/payee_locations", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(200).set_body_json(payee_locations_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let locations = client
            .get_payee_locations(PlanId::Id(uuid!(TEST_ID_1)))
            .await
            .unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].id.to_string(), TEST_ID_4);
    }

    #[tokio::test]
    async fn get_payee_locations_by_payee_returns_locations() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/payees/{}/payee_locations",
                TEST_ID_1, TEST_ID_3
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(payee_locations_list_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let locations = client
            .get_payee_locations_by_payee(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_3))
            .await
            .unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].payee_id.to_string(), TEST_ID_3);
    }

    #[tokio::test]
    async fn get_payee_location_returns_location() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/plans/{}/payee_locations/{}",
                TEST_ID_1, TEST_ID_4
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(payee_location_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let location = client
            .get_payee_location(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_4))
            .await
            .unwrap();
        assert_eq!(location.id.to_string(), TEST_ID_4);
        assert_eq!(location.latitude, "37.7749");
    }

    #[tokio::test]
    async fn create_payee_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("POST"))
            .and(path(format!("/plans/{}/payees", TEST_ID_1)))
            .respond_with(ResponseTemplate::new(201).set_body_json(payee_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (payee, sk) = client
            .create_payee(
                PlanId::Id(uuid!(TEST_ID_1)),
                PostPayee {
                    name: "Amazon".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(payee.id.to_string(), TEST_ID_3);
        assert_eq!(sk, 3);
    }

    #[tokio::test]
    async fn update_payee_succeeds() {
        let (client, server) = new_test_client().await;
        Mock::given(method("PATCH"))
            .and(path(format!("/plans/{}/payees/{}", TEST_ID_1, TEST_ID_3)))
            .respond_with(ResponseTemplate::new(200).set_body_json(payee_single_fixture()))
            .expect(1)
            .mount(&server)
            .await;
        let (payee, _) = client
            .update_payee(
                PlanId::Id(uuid!(TEST_ID_1)),
                uuid!(TEST_ID_3),
                SavePayee {
                    name: Some("Amazon Updated".to_string()),
                },
            )
            .await
            .unwrap();
        assert_eq!(payee.id.to_string(), TEST_ID_3);
    }

    #[tokio::test]
    async fn get_payee_returns_not_found() {
        let (client, server) = new_test_client().await;
        Mock::given(method("GET"))
            .and(path(format!("/plans/{}/payees/{}", TEST_ID_1, TEST_ID_3)))
            .respond_with(ResponseTemplate::new(404).set_body_json(error_body(
                "404",
                "not_found",
                "Payee not found",
            )))
            .mount(&server)
            .await;
        let err = client
            .get_payee(PlanId::Id(uuid!(TEST_ID_1)), uuid!(TEST_ID_3))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }
}
