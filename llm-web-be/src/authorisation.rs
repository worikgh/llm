use crate::data_store::get_user_records;
use crate::session::Session;
use base64::{engine::general_purpose, Engine as _};
use bcrypt::verify;
use chrono::DateTime;
use chrono::{Duration, NaiveDateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use simple_crypt::decrypt;
use simple_crypt::encrypt;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
const SESSION_EXP: i64 = 2; // Hours a session lasts for if incactive
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
/// Hierarchical.  Admin has all rights.  Chat can chat, NoRights....
pub enum UserRights {
    NoRights,
    Chat,
    Admin,
}
impl fmt::Display for UserRights {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserRights::NoRights => f.write_str("NoRights"),
            UserRights::Chat => f.write_str("Chat"),
            UserRights::Admin => f.write_str("Admin"),
        }
    }
}
/// Get a list of user records
pub async fn users() -> io::Result<Vec<AuthorisationRecord>> {
    Ok(get_user_records().await?.to_vec())
}

/// Returned to caller on successful login
#[derive(Debug)]
pub struct LoginResult {
    //pub rights: UserRights,
    pub uuid: Uuid,
    pub token: String, // Send this back to user.  It must be sent with every request
    pub expiry: DateTime<Utc>,
}

/// Get the next expiry time.  Now plus a constant
pub fn next_expire() -> DateTime<Utc> {
    Utc::now() + Duration::hours(SESSION_EXP)
}

/// Check if a user is authorised with `password`.  If so create an
/// entry in the session database and return a `LoginResult` object
/// for them.  If they are not authorised return None.
pub async fn login(
    username: String,
    password: String,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
) -> io::Result<Option<LoginResult>> {
    // Process array of `AuthorisationRecord`
    let records: Vec<AuthorisationRecord> = get_user_records().await?;

    match records.iter().find(|&x| x.username == username) {
        Some(record) => {
            // TODO: Is this forced unwrap OK?  What about perverse
            // passwords?  Must sanatise passwords so cannot get
            // control characters
            if verify(&password, &(record.password)).unwrap() {
                // Successful login.
                // Initialise session and a result
                let expiry: DateTime<Utc> = next_expire();
                let key = record.key.clone();
                let uuid: Uuid = record.uuid;
                let token = generate_token(&uuid, &expiry, &key);
                let level = record.level;
                sessions.lock().unwrap().insert(
                    token.clone(),
                    Session::new(record.uuid, expiry, token.clone(), record.credit, level),
                );
                Ok(Some(LoginResult {
                    //rights: record.level,
                    uuid,
                    token,
                    expiry,
                }))
            } else {
                // Failed login.  Not an error
                println!(
                    "login({username}, {password}) Failed verify: {} ",
                    record.password
                );
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

/// The data stored about a user.
/// The `name`, and  `password` are supplied by the user
/// The `uuid` is used to identify a user
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthorisationRecord {
    pub username: String,
    pub password: String,
    pub uuid: Uuid,
    pub level: UserRights,
    pub credit: f64,
    pub key: Vec<u8>,
}

impl fmt::Display for AuthorisationRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "username: {}\n\tlevel: {}\n\tcredit: {:0.3}\n",
            self.username, self.level, self.credit,
        )
    }
}
/// Handle tokens.  Tokens are made from a uuid and session expiry
/// time.  The uuid and expiry can be recovered.  (No use for that yet)
pub fn generate_token(uuid: &Uuid, expiry: &DateTime<Utc>, key: &[u8]) -> String {
    general_purpose::STANDARD
        .encode(encrypt(format!("{uuid}{expiry}").as_bytes(), key).expect("Encrypt a token"))
}

#[allow(dead_code)]
pub fn decode_token(
    encoded_uuid_expiry: String,
    key: &[u8],
) -> Result<(Uuid, DateTime<Utc>), Box<dyn std::error::Error>> {
    let decoded_data = general_purpose::STANDARD.decode(encoded_uuid_expiry)?;
    let decrypted_data = decrypt(&decoded_data, key)?;

    let decrypted_string = String::from_utf8(decrypted_data)?;

    let parts: (&str, &str) = decrypted_string.split_at(36);
    let uuid_part = parts.0;
    let datetime_part = parts.1;

    let uuid = Uuid::parse_str(uuid_part)?;
    let datetime = DateTime::<Utc>::from_naive_utc_and_offset(
        NaiveDateTime::parse_from_str(datetime_part, "%Y-%m-%d %H:%M:%S%.f %Z")?,
        Utc,
    );

    Ok((uuid, datetime))
}

#[cfg(test)]
pub mod tests {
    // //use llm_web_common::communication::LoginRequest;

    use super::*;

    #[tokio::test]
    async fn test_login() {
        use crate::data_store::add_user;
        use crate::data_store::delete_user;
        use crate::data_store::tests::get_unique_user;
        let username = get_unique_user("authorisation::tests::test_login").await;
        let password = "123";
        let b: bool = add_user(username.as_str(), "123").await.unwrap();

        assert!(b);

        // Test logging the user in
        let sessions = Arc::new(Mutex::new(HashMap::<String, Session>::new()));
        let test: bool = match login(username.clone(), password.to_string(), sessions).await {
            Ok(t) => t.is_some(),
            Err(err) => panic!("{}", err),
        };
        // Test can log user in
        assert!(test);
        assert!(delete_user(username.as_str()).await.unwrap());
    }
    // #[tokio::test]
    #[test]
    fn token_coding() {
        let uuid = Uuid::new_v4();
        let expiry: DateTime<Utc> = Utc::now() + Duration::hours(SESSION_EXP);
        let key: Vec<u8> = vec![1, 2, 3, 4];
        let token = generate_token(&uuid, &expiry, &key);

        match NaiveDateTime::parse_from_str(
            "2023-09-10 07:31:29.249939359 UTC",
            "%Y-%m-%d %H:%M:%S%.f %Z",
        ) {
            Ok(_) => (),
            Err(err) => panic!("Failed time: {}", err),
        };

        let (uuid_test, expiry_test) = match decode_token(token, &key) {
            Ok(a) => a,
            Err(err) => panic!("{}", err),
        };
        assert!(uuid == uuid_test);
        assert!(expiry == expiry_test);
    }
}
