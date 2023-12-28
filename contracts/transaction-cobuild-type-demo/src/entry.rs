use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    high_level::{load_cell_data, load_script_hash},
    syscalls::SysError,
};
use ckb_transaction_cobuild::fetch_message;
use core::result::Result;

use crate::error::Error;

pub fn main() -> Result<(), Error> {
    // fetch the message field of SighashAll and verify it
    if let Ok(Some(message)) = fetch_message() {
        let script_hash = load_script_hash()?;
        for action in message.actions().into_iter() {
            if action.script_hash().as_slice() == script_hash.as_slice() {
                if !verify_action_data(&action.data().raw_data())? {
                    return Err(Error::InvalidMessage);
                }
            }
        }
    }
    // other cell data verification logic goes here
    Ok(())
}

// a simple verification logic for the demo
fn verify_action_data(data: &[u8]) -> Result<bool, Error> {
    if data == b"MINT" {
        if let Err(SysError::IndexOutOfBound) = load_cell_data(0, Source::GroupInput) {
            return Ok(!load_cell_data(0, Source::GroupOutput)?.is_empty());
        }
    } else if data == b"BURN" {
        if let Err(SysError::IndexOutOfBound) = load_cell_data(0, Source::GroupOutput) {
            return Ok(!load_cell_data(0, Source::GroupInput)?.is_empty());
        }
    }
    Ok(false)
}
