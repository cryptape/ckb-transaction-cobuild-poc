pub use blake2b_ref::{Blake2b, Blake2bBuilder};
pub use molecule::lazy_reader::Cursor;

pub const PERSONALIZATION_SIGHASH_ALL: &[u8] = b"ckb-tcob-sighash";
pub const PERSONALIZATION_SIGHASH_ALL_ONLY: &[u8] = b"ckb-tcob-sgohash";
pub const PERSONALIZATION_OTX: &[u8] = b"ckb-tcob-otxhash";

const BATCH_SIZE: usize = 2048;

/// return a blake2b instance with personalization for SighashAll
pub fn new_sighash_all_blake2b() -> Blake2bStatistics {
    Blake2bStatistics::new(
        Blake2bBuilder::new(32)
            .personal(PERSONALIZATION_SIGHASH_ALL)
            .build(),
    )
}

/// return a blake2b instance with personalization for SighashAllOnly
pub fn new_sighash_all_only_blake2b() -> Blake2bStatistics {
    Blake2bStatistics::new(
        Blake2bBuilder::new(32)
            .personal(PERSONALIZATION_SIGHASH_ALL_ONLY)
            .build(),
    )
}

/// return a blake2b instance with personalization for OTX
pub fn new_otx_blake2b() -> Blake2bStatistics {
    Blake2bStatistics::new(
        Blake2bBuilder::new(32)
            .personal(PERSONALIZATION_OTX)
            .build(),
    )
}

pub struct Blake2bStatistics {
    count: usize,
    blake2b: Blake2b,
}

impl Blake2bStatistics {
    pub fn new(blake2b: Blake2b) -> Self {
        Self { count: 0, blake2b }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.blake2b.update(data);
        self.count += data.len();
    }
    pub fn update_cursor(&mut self, mut cursor: Cursor) {
        let mut buf = [0u8; BATCH_SIZE];
        while cursor.size > 0 {
            let read_len = cursor.read_at(&mut buf).unwrap();
            if read_len > 0 {
                self.update(&buf[0..read_len]);
                cursor = cursor.slice_by_start(read_len).unwrap();
            }
        }
    }

    pub fn finalize(self, dst: &mut [u8]) {
        self.blake2b.finalize(dst)
    }
    pub fn count(&self) -> usize {
        self.count
    }
}
