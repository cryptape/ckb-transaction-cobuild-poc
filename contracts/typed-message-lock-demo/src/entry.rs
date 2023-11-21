use core::result::Result;

// Import CKB syscalls and structures
// https://docs.rs/ckb-std/
use ckb_std::{
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    high_level::load_script,
};

use crate::error::Error;
use ckb_auth_rs::{
    ckb_auth::{ckb_auth, CkbEntryType},
    AuthAlgorithmIdType, CkbAuthType, EntryCategoryType,
};

pub fn main() -> Result<(), Error> {
    let message = [0u8; 32];
    let signature = [0u8; 65];
    let mut pubkey_hash = [0u8; 20];
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    pubkey_hash.copy_from_slice(&args[0..20]);

    let id = CkbAuthType {
        algorithm_id: AuthAlgorithmIdType::Ckb,
        pubkey_hash: pubkey_hash,
    };

    let entry = CkbEntryType {
        code_hash: [0u8; 32],
        hash_type: ScriptHashType::Data1,
        entry_category: EntryCategoryType::DynamicLinking,
    };

    ckb_auth(&entry, &id, &signature, &message).map_err(|_| Error::AuthError)?;

    Ok(())
}
