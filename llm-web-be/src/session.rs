/// User Session.  One is generated on login and deleted on log off or on expiry
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::authorisation::UserRights;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Session {
    pub uuid: Uuid,            // Id user
    pub expire: DateTime<Utc>, // When session expires
    pub token: String,         // Encrypted token user must pass to use session
    pub credit: f64,           // Fractions of a cent. LLM is cheep
    pub level: UserRights,
}

impl Drop for Session {
    fn drop(&mut self) {
        eprintln!("Dropping session.  Credit: {:0.5}", self.credit);
    }
}

impl Session {
    pub fn new(
        uuid: Uuid,
        expire: DateTime<Utc>,
        token: String,
        credit: f64,
        level: UserRights,
    ) -> Self {
        eprintln!("Create session: {uuid}: Credit: {credit}");
        Self {
            uuid,
            expire,
            token,
            credit,
            level,
        }
    }
    pub fn set_expire(&mut self, new_expire: DateTime<Utc>) {
        self.expire = new_expire;
    }
}
