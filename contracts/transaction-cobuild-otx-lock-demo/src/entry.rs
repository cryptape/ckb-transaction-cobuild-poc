use ckb_std::{
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    high_level::load_script,
};
use ckb_transaction_cobuild::{cobuild_entry, Callback};
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

struct Verifier {
    entry: CkbEntryType,
    id: CkbAuthType,
}

impl Verifier {
    pub fn new(entry: CkbEntryType, id: CkbAuthType) -> Self {
        Self { entry, id }
    }
}

impl Callback for Verifier {
    fn invoke(
        &self,
        seal: &[u8],
        signing_message_hash: &[u8; 32],
    ) -> Result<(), ckb_transaction_cobuild::error::Error> {
        let auth_result = ckb_auth(&self.entry, &self.id, seal, signing_message_hash);
        match auth_result {
            Ok(_) => Ok(()),
            Err(_) => Err(ckb_transaction_cobuild::error::Error::AuthError),
        }
    }
}

pub fn main() -> Result<(), Error> {
    let mut pubkey_hash = [0u8; 20];
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
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

    let verifier = Verifier::new(entry, id);
    let verify_pass = cobuild_entry(verifier)?;
    if verify_pass {
        Ok(())
    } else {
        return Err(Error::AuthError);
    }
}
