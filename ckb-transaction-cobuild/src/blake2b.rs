pub use blake2b_ref::{Blake2b, Blake2bBuilder};

pub const PERSONALIZATION_SIGHASH_ALL: &[u8] = b"ckb-tcob-sighash";
pub const PERSONALIZATION_SIGHASH_ALL_ONLY: &[u8] = b"ckb-tcob-sgohash";
pub const PERSONALIZATION_OTX: &[u8] = b"ckb-tcob-otxhash";

/// return a blake2b instance with personalization for SighashAll
pub fn new_sighash_all_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(PERSONALIZATION_SIGHASH_ALL)
        .build()
}

/// return a blake2b instance with personalization for SighashAllOnly
pub fn new_sighash_all_only_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(PERSONALIZATION_SIGHASH_ALL_ONLY)
        .build()
}

/// return a blake2b instance with personalization for OTX
pub fn new_otx_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(PERSONALIZATION_OTX)
        .build()
}
