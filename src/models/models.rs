use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonInfo {
    pub header: String,
    pub body: String
}