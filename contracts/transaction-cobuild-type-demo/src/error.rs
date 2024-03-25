use ckb_std::error::SysError;
use ckb_transaction_cobuild::error::Error as CobuildError;
use ckb_transaction_cobuild::error::LazyReaderError;

/// Error
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    InvalidMessage,
    CobuildError,
    LazyReaderError,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl From<CobuildError> for Error {
    fn from(_: CobuildError) -> Self {
        Self::CobuildError
    }
}

impl From<LazyReaderError> for Error {
    fn from(_: LazyReaderError) -> Self {
        Self::LazyReaderError
    }
}
