// use llm_rs;
/// Structures to send back and forth between llm-web-fe and llm-web-be utilises
//use rsa::RsaPublicKey;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
//use uuid::serde;
use uuid::Uuid;
/// The communication between the front end and the back end uses
/// `Message` struct.  `CommType` categorises the communication and
/// defines what object is being relayed in the `Message.object` type
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum CommType {
    LoginRequest,
    LoginResponse,
    LogoutRequest,
    LogoutResponse,
    ChatPrompt,
    ChatResponse,
    InvalidRequest,
}

/// The messae as sent: `comm_type` says what type it is, the String
/// is encoded JSON of the object itself.  
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub comm_type: CommType,
    pub object: String,
}

/// Server -> Client.  
#[derive(Debug, Deserialize, Serialize)]
pub struct InvalidRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub uuid: Option<Uuid>,
    pub token: Option<String>,
    pub credit: f64,
    pub expire: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutRequest {
    pub uuid: Uuid,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutResponse {
    pub success: bool, // Will only fail if not logged in (FLW)
}

// From llm-web-fe -> llm-web-be.  A prompt for a chat session.  Sent
// from front end to server.
// The message sent to OpenAI looks like:
//   -d '{
//      "model": "gpt-3.5-turbo",
//      "messages": [{"role": "user", "content": "Say this is a test!"}],
//      "temperature": 0.7
//    }'
/// Each mesage has a type
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LLMMessageType {
    System,    // First message:  What attitude the LLM should take
    User,      // Directed to the LLM
    Assistant, // Response from the LLM
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LLMMessage {
    pub role: LLMMessageType,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ChatPrompt {
    /// The model to use    
    pub model: String,

    // The
    pub messages: Vec<LLMMessage>,

    pub temperature: f64,

    // The user's authenticating data
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExtraInfo {
    /// The OpenAI response headers.  Added after it is received
    pub headers: HashMap<String, String>,

    /// The time the request took in milli seconds
    pub duration: u128,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// From llm-web-be -> llm-web-fe.  Response from LLM
/// Has to send back all the information the front end needs
pub struct ChatResponse {
    // The remaining credit the user has
    pub credit: f64,

    // For every chat response there is a (possibly zero) cost.  Not
    // contemplating a negative cost, but it is possible.
    pub cost: f64,

    // The response: OpenAI can return an array of responses,
    // essentially offering many opinions.  This is controlled with a
    // parameter in the request.  Here, for now, only one will be
    // requested by the front end.  OpenAI returns a "role" and
    // "content".  The "role" is always 'assistant' (test this!).  SO
    // just pass back one string for the response
    pub response: String,

    /// The model that the LLM used.  It might not be the model asked for.
    pub model: String,

    /// The data about the hat that the backend sends the front end
    pub backend_data: Option<ExtraInfo>,
}

// Display for CommType
impl fmt::Display for CommType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommType::LoginRequest => write!(f, "Login Request"),
            CommType::LoginResponse => write!(f, "Login Response"),
            CommType::LogoutRequest => write!(f, "Logout Request"),
            CommType::LogoutResponse => write!(f, "Logout Response"),
            CommType::ChatPrompt => write!(f, "Chat Prompt"),
            CommType::ChatResponse => write!(f, "Chat Response"),
            CommType::InvalidRequest => write!(f, "Invalid Request"),
        }
    }
}

// Display for Message
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "comm_type: {}, object: {}", self.comm_type, self.object)
    }
}

// Display for LoginRequest
impl fmt::Display for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "username: {}, password: {}",
            self.username, self.password
        )
    }
}

// Display for ChatPrompt
impl fmt::Display for ChatPrompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "model: {}", self.model)
    }
}

// Display for LoginResponse
impl fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Success: {} ", self.success,)
    }
}

// Write a `from` method for every struct that is not `Message`
// converting them to a `Message`.  Use `serde_json::to_string` to
// initialise the `object` fields.
impl From<InvalidRequest> for Message {
    fn from(request: InvalidRequest) -> Self {
        Message {
            comm_type: CommType::InvalidRequest,
            object: serde_json::to_string(&request).unwrap(),
        }
    }
}

impl From<LoginRequest> for Message {
    fn from(request: LoginRequest) -> Self {
        Message {
            comm_type: CommType::LoginRequest,
            object: serde_json::to_string(&request).unwrap(),
        }
    }
}

impl From<ChatPrompt> for Message {
    fn from(prompt: ChatPrompt) -> Self {
        Message {
            comm_type: CommType::ChatPrompt,
            object: serde_json::to_string(&prompt).unwrap(),
        }
    }
}

impl From<LoginResponse> for Message {
    fn from(response: LoginResponse) -> Self {
        Message {
            comm_type: CommType::LoginResponse,
            object: serde_json::to_string(&response).unwrap(),
        }
    }
}

impl From<LogoutRequest> for Message {
    fn from(request: LogoutRequest) -> Self {
        Message {
            comm_type: CommType::LogoutRequest,
            object: serde_json::to_string(&request).unwrap(),
        }
    }
}
impl From<LogoutResponse> for Message {
    fn from(response: LogoutResponse) -> Self {
        Message {
            comm_type: CommType::LogoutResponse,
            object: serde_json::to_string(&response).unwrap(),
        }
    }
}
