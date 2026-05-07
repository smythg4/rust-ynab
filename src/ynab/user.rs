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

/// User represents the authenticated YNAB user.
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
