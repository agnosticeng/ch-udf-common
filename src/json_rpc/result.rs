use std::fmt::Debug;
use thiserror::Error;
use serde_json::Value;
use alloy::transports::{RpcError, TransportErrorKind};
use alloy_json_rpc::ErrorPayload;

pub type Result<T> = std::result::Result<T, JSONRPCError>;

#[derive(Error, Debug)]
pub enum JSONRPCError {
    #[error("URL parse error")]
    URLParse(#[from] url::ParseError),
    #[error("URL parameters parse error")]
    URLParamsParse(#[from] serde_qs::Error),
    #[error("Transport error")]
    Transport(#[from] RpcError<TransportErrorKind>),
    #[error("payload")]
    Payload(ErrorPayload),
    #[error("null value")]
    NullValue
}

pub type JSONRPCCallResult = std::result::Result<serde_json::Value, JSONRPCError>;

pub struct JSONRPCCall {
    pub method: String,
    pub params: Value
}

pub type BatchJSONRPCResult = Result<Vec<JSONRPCCallResult>>;