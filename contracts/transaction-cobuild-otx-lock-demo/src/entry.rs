use ckb_std::{
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    high_level::{load_script, load_script_hash},
};
use ckb_transaction_cobuild::parse_otx_message;
use core::result::Result;

use crate::error::Error;
use ckb_auth_rs::{
    ckb_auth::{ckb_auth, CkbEntryType},
    AuthAlgorithmIdType, CkbAuthType, EntryCategoryType,
};

const AUTH_CODE_HASH: [u8; 32] = [
    0x90, 0x17, 0xda, 0xdb, 0x54, 0x93, 0xe6, 0x31, 0x7d, 0xa3, 0xba, 0xb8, 0xa1, 0x45, 0x68, 0x51,
    0xd4, 0x50, 0x43, 0xff, 0x70, 0x1d, 0x64, 0x55, 0xa0, 0x3a, 0xbd, 0xab, 0xca, 0xd9, 0x9e, 0x3e,
];

pub fn main() -> Result<(), Error> {
    let mut pubkey_hash = [0u8; 20];
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    let script_hash = load_script_hash()?;
    pubkey_hash.copy_from_slice(&args[0..20]);

    let id = CkbAuthType {
        algorithm_id: AuthAlgorithmIdType::Ckb,
        pubkey_hash,
    };

    let entry = CkbEntryType {
        code_hash: AUTH_CODE_HASH,
        hash_type: ScriptHashType::Data1,
        entry_category: EntryCategoryType::DynamicLinking,
    };

    let mut found = false;
    for (message_digest, seals) in parse_otx_message()? {
        for seal_pair in seals {
            if seal_pair.script_hash().as_slice() == script_hash.as_slice() {
                ckb_auth(&entry, &id, &seal_pair.seal().raw_data(), &message_digest)
                    .map_err(|_| Error::AuthError)?;
                found = true;
            }
        }
    }

    if found {
        Ok(())
    } else {
        Err(Error::AuthError)
    }
}
