use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ynab::client::Client;
use crate::ynab::common::NO_PARAMS;
use crate::ynab::errors::Error;

#[derive(Debug, Deserialize)]
struct UserDataEnvelope {
    data: UserData,
}

#[derive(Debug, Deserialize)]
struct UserData {
    user: User,
}

/// The authenticated YNAB user.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
}

impl Client {
    /// Returns authenticated user information.
    pub async fn get_user(&self) -> Result<User, Error> {
        let result: UserDataEnvelope = self.get("user", NO_PARAMS).await?;
        Ok(result.data.user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ynab::testutil::{TEST_ID_3, error_body, new_test_client, user_fixture};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    fn user_single_fixture() -> serde_json::Value {
        serde_json::json!({
            "data": { "user": user_fixture() }
        })
    }
    #[tokio::test]
    async fn get_user_returns_id() {
        let (client, server) = new_test_client().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!(user_single_fixture())),
            )
            .expect(1)
            .mount(&server)
            .await;

        let user = client.get_user().await.unwrap();
        assert_eq!(user.id.to_string(), TEST_ID_3);
    }

    #[tokio::test]
    async fn get_user_returns_unauthorized() {
        let (client, server) = new_test_client().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(401).set_body_json(error_body(
                "401",
                "unauthorized",
                "Unauthorized",
            )))
            .mount(&server)
            .await;

        let err = client.get_user().await.unwrap_err();
        assert!(matches!(err, Error::Unauthorized(_)));
    }
}
