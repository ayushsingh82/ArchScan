// XXX: Remove this allow if someone can migrate
// our cosmwasm_storage functions to cw-storage-plus
// (fyi it's a monumental task)
#![allow(deprecated)]

pub mod contract;
pub mod handlers;
pub mod msg;
pub mod read_utils;
pub mod state;
pub mod write_utils;

mod error;
mod integration_test;

pub use crate::error::ContractError;
