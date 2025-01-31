//! The backend server for llm-web.  Handles the communications with LLM
extern crate bcrypt;
extern crate llm_rs;
extern crate llm_web_common;
mod authorisation;
mod server;
mod session;
use std::env;
mod data_store;
//use async_std::task;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
// main
#[tokio::main]
async fn main() {
    let mut args: env::Args = env::args();
    let _programme_name = args.next().unwrap();

    // If one argument "test", then the LLM is not contacted
    let mut args: Vec<String> = args.collect();
    if args.is_empty() || args[0] == "test" {
        if let Err(e) = server::AppBackend::run_server().await {
            eprintln!("FAILED: {}", e);
            std::process::exit(1);
        }

        std::process::exit(0);
    }
    // Make the args back to being a iterator for convenience
    let mut args = args.iter_mut();
    let s1 = args.next().unwrap();
    const USAGE: &str = "Usage: adduser <username> <password>";
    match s1.as_str() {
        "delete_user" => {
            let username = args.next().expect(USAGE);
            match data_store::delete_user(username.as_str()).await {
                Ok(b) => {
                    if b {
                        println!("Deleted: {username}");
                    } else {
                        println!("Not found: {username}");
                    }
                }
                Err(err) => eprintln!("Failed to delete user {}: {}", username, err),
            };
        }
        "add_user" => {
            let username = args.next().expect(USAGE);
            let password: String = args.fold(String::new(), |a, b| format!("{a} {b}"));
            match data_store::add_user(username.as_str(), password.as_str()).await {
                Ok(b) => {
                    if b {
                        println!("Added: {username}");
                    } else {
                        println!("Already added: {username}");
                    }
                }

                Err(err) => eprintln!("Adding user failed: {err}"),
            };
        }

        "list_users" => {
            match authorisation::users().await {
                Ok(users) => {
                    for u in users {
                        println!("{u}");
                    }
                }

                Err(err) => panic!("{}", err),
            };
        }
        _ => panic!("{}\nNot: '{}'", USAGE, s1),
    };
}
