/// The user data
use crate::authorisation::UserRights;
//use crate::session::Session;
// use std::fs::File;
// use std::io;
// use std::io::SeekFrom;
use crate::authorisation::AuthorisationRecord;
// use crate::session::Session;
// use base64::{engine::general_purpose, Engine as _};
use bcrypt::{hash, DEFAULT_COST};
// use chrono::DateTime;
// use chrono::{Duration, NaiveDateTime, Utc};
use fs2::FileExt;
use rand::Rng;
// use serde::Deserialize;
// use serde::Serialize;
// use simple_crypt::decrypt;
// use simple_crypt::encrypt;
// use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
// use std::sync::{Arc, Mutex};
use uuid::Uuid;
const FILENAME: &str = "users.txt";

/// This is the main gateway into the data store.  The data is in a
/// JSON file, and only one process can access it at a time.  This
/// returns a opened, read/write locked file handle.
/// This is very simple.  
fn get_locked_handle() -> io::Result<File> {
    let file = match OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(FILENAME)
    {
        Ok(f) => f,
        Err(err) => panic!("{}: Filename: {}", err, FILENAME),
    };
    file.lock_exclusive()?;
    Ok(file)
}

/// Remove a users record
/// Return
/// * `true` if record deleted
/// * `false if record not found
pub async fn delete_user(username: &str) -> io::Result<bool> {
    let username = username.to_string();
    tokio::task::spawn_blocking(move || -> io::Result<bool> {
        let mut file: File = get_locked_handle()?;
        file.seek(SeekFrom::Start(0))?;

        // Got file of data locked and ready to read
        // Read it into `contents` and transform to records
        let mut contents = String::new();
        for line in BufReader::new(&file).lines() {
            contents += line?.as_str();
        }
        let mut records: Vec<AuthorisationRecord> = if contents.is_empty() {
            // No users yet
            return Ok(false);
        } else {
            match serde_json::from_str(contents.as_str()) {
                Ok(s) => s,
                Err(err) => panic!("{}", err),
            }
        };

        // Search for user record. If it is there delete it and over
        // write the user file
        if let Some(pos) = records.iter().position(|x| x.username == username) {
            records.remove(pos);

            let contents = serde_json::to_string(&records)?;
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            let mut fw = BufWriter::new(file);
            fw.write_all(contents.as_bytes())?;
            Ok(true)
        } else {
            // Not found
            Ok(false)
        }
    })
    .await?
}

/// Add a user to the system.  Set them up with a record in the
/// Authorisation DB with: User name, password, UUID, encryption key,
/// and UserRights.  (The encryption key is used to encrypt thier
/// session tokens).  Return false if already in the system.  True
/// otherwise
pub async fn add_user(username: &str, password: &str) -> io::Result<bool> {
    // No white space in passwords
    let hashed_password = hash(password.trim(), DEFAULT_COST).unwrap();
    let rng = rand::thread_rng();
    let key: Vec<u8> = rng
        .sample_iter(rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
        .as_bytes()
        .to_vec();

    let auth_rec = AuthorisationRecord {
        username: username.to_string(),
        level: UserRights::Chat,
        password: hashed_password,
        uuid: Uuid::new_v4(),
        key,
        credit: 0.0,
    };

    tokio::task::spawn_blocking(move || -> io::Result<bool> {
        let mut file = get_locked_handle()?;
        file.seek(SeekFrom::Start(0))?;
        let lines = BufReader::new(&file).lines();
        let mut contents = String::new();
        for line in lines {
            contents += line?.as_str();
        }

        let mut records: Vec<AuthorisationRecord> = if contents.is_empty() {
            // No users yet
            vec![]
        } else {
            match serde_json::from_str(contents.as_str()) {
                Ok(s) => s,
                Err(err) => panic!("{}", err),
            }
        };

        if records.iter().any(|x| x.username == auth_rec.username) {
            // Record exists
            // `false` means do not need to add user
            // Do not overwrite an existing user
            // Explicitly delete it first
            Ok(false)
        } else {
            records.push(auth_rec);
            let contents = serde_json::to_string(&records)?;
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            let mut fw = BufWriter::new(file);
            fw.write_all(contents.as_bytes())?;
            Ok(true)
        }
    })
    .await?
}

/// Get all the authorisation records for a read only purpose
pub async fn get_user_records() -> io::Result<Vec<AuthorisationRecord>> {
    let mut file: File = get_locked_handle()?;
    let mut s = String::new();
    file.seek(SeekFrom::Start(0))?;
    let lines = BufReader::new(file).lines();
    for line in lines {
        s += line?.as_str();
    }
    if s.is_empty() {
        return Ok(vec![]);
    }
    let records: Vec<AuthorisationRecord> = match serde_json::from_str(s.as_str()) {
        Ok(v) => v,
        Err(err) => panic!("{}: Failed to decode {}", err, s),
    };
    Ok(records)
}

#[allow(unused)]
/// The AuthorisationRecord for a user.
pub async fn get_user(uuid: Uuid) -> io::Result<AuthorisationRecord> {
    Ok(get_user_records()
        .await
        .unwrap()
        .iter()
        .find(|&x| x.uuid == uuid)
        .unwrap()
        .clone())
}

/// Save the user data out of a Session  
/// Precondition: User identified by `session` must exist
#[allow(unused)]
pub async fn update_user(uuid: Uuid, credit: f64, level: UserRights) -> io::Result<()> {
    tokio::task::spawn_blocking(move || -> io::Result<()> {
        let mut file = get_locked_handle()?;
        let lines = BufReader::new(&file).lines();
        let mut contents = String::new();
        for line in lines {
            contents += line?.as_str();
        }

        let mut records: Vec<AuthorisationRecord> = if contents.is_empty() {
            // No users yet
            panic!("update_user(..): No users");
        } else {
            match serde_json::from_str(contents.as_str()) {
                Ok(s) => s,
                Err(err) => panic!("{}", err),
            }
        };

        match records.iter_mut().find(|a| a.uuid == uuid) {
            Some(r) => {
                r.credit = credit;
                r.level = level;
            }
            None => panic!("update_user(..): User does not exist in database",),
        };

        let contents = serde_json::to_string(&records)?;
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        let mut fw = BufWriter::new(file);
        fw.write_all(contents.as_bytes())
    })
    .await?
}

#[cfg(test)]
pub mod tests {

    use super::*;
    #[tokio::test]
    async fn test_get_user_records() {
        assert!(match get_user_records().await {
            Ok(_) => true,
            Err(err) => panic!("Error: {}, ", err),
        });
    }
    #[tokio::test]
    async fn test_user() {
        //use chrono::Utc;
        let username = get_unique_user("test_user").await;
        assert!(!get_user_records()
            .await
            .unwrap()
            .iter()
            .any(|x| x.username == username));
        let password = "123";
        let b: bool = add_user(username.as_str(), password).await.unwrap();
        assert!(b, "Failed add_user({}, {})", username, password);
        let uuid = get_user_records()
            .await
            .unwrap()
            .iter()
            .find(|&x| x.username == username)
            .unwrap()
            .uuid;
        let auth_record = get_user(uuid).await.unwrap();
        assert!(uuid == auth_record.uuid);
        assert!(auth_record.level == UserRights::Chat);
        assert!(auth_record.credit == 0.0);
        // Test updating a user
        let test_credit: f64 = 3.45;
        let test_level = UserRights::Admin;
        // let session = Session {
        //     uuid,
        //     expire: Utc::now(),
        //     token: "Not part of test".to_string(),
        //     credit: test_credit, // Testing this
        //     level: test_level,   // Testing this
        // };
        update_user(uuid, test_credit, test_level).await.unwrap();
        let auth_record = get_user(uuid).await.unwrap();
        assert!(uuid == auth_record.uuid);
        assert!(auth_record.level == test_level);
        assert!(auth_record.credit == test_credit);

        let b = delete_user(username.as_str()).await.unwrap();
        assert!(b);
        assert!(!get_user_records()
            .await
            .unwrap()
            .iter()
            .any(|x| x.username == username));
    }
    pub async fn get_unique_user(pfx: &str) -> String {
        let user_list: Vec<String> = get_user_records()
            .await
            .unwrap()
            .iter()
            .map(|x| x.username.clone())
            .collect();

        let mut name_pfx = pfx.to_string();
        let letters: Vec<char> = (b'a'..=b'z').map(char::from).collect();
        let mut itr = letters.iter().peekable();
        let mut itr2 = letters.iter();
        let mut test_name: String;
        loop {
            test_name = format!("{name_pfx}_{}", itr.next().unwrap());
            if !user_list.contains(&test_name) {
                break;
            }
            if itr.peek().is_none() {
                name_pfx = format!("{name_pfx}{}_", itr2.next().unwrap());
            }
        }
        test_name
    }
}
