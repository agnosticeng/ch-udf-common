use anyhow::anyhow;
use core::str;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::clone::Clone;
use std::fmt::Debug;
use std::fmt::Display;
use std::result::Result;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum JSONResult {
    #[serde(rename = "value")]
    Value(Value),
    #[serde(rename = "error")]
    Error(String),
}

impl JSONResult {
    pub fn empty() -> JSONResult {
        JSONResult::Value(serde_json::json!({}))
    }
}

impl<T: serde::Serialize, E: Display> From<Result<T, E>> for JSONResult {
    fn from(res: Result<T, E>) -> Self {
        match res {
            Err(e) => JSONResult::Error(e.to_string()),
            Ok(v) => match serde_json::to_value(v) {
                Err(e) => JSONResult::Error(e.to_string()),
                Ok(js) => JSONResult::Value(js),
            },
        }
    }
}

impl From<JSONResult> for anyhow::Result<Value> {
    fn from(res: JSONResult) -> Self {
        match res {
            JSONResult::Error(e) => Err(anyhow!(e)),
            JSONResult::Value(v) => Ok(v),
        }
    }
}
