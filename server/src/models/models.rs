#![allow(dead_code)]
use std::fmt;
use serde::{Serialize, Deserialize};

//JsonInfo -----------------------------------------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonInfo {
    pub header: String,
    pub body: String
}

impl JsonInfo {
    pub fn new() -> Self {
        JsonInfo { header: String::new(), body: String::new() }
    }

    pub fn from(header: &str, body: &str) -> Self {
        JsonInfo { header: header.to_string(), body: body.to_string() }
    }

    pub fn from_str(header: String, body: String) -> Self {
        JsonInfo { header, body }
    }
 
    pub fn is_empty(&self) -> bool {
        self.header.is_empty() && self.body.is_empty()
    }

    pub fn clear(&mut self) {
        self.header.clear();
        self.body.clear();
    }
}

impl fmt::Display for JsonInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::from("{");

        if !self.is_empty() {
            if !self.header.is_empty() {
                s = format!("{}\n\theader: {}", s, self.header);
            }
            if !self.body.is_empty() {
                s = format!("{}\n\tbody: {}", s, self.body);
            }
            s = format!("{}\n}}", s);
        } else {
            s = String::from("{}");
        }
        
        write!(f, "{}", s)
    }
}