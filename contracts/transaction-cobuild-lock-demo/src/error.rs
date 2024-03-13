use ckb_std::error::SysError;
use ckb_transaction_cobuild;

/// Error
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    AuthError,
    WrongSighashAll,
    WrongWitnessLayout,
    WrongOtxStart,
    WrongOtx,
    NoSealFound,
    ScriptHashAbsent,
    WrongCount,
    LazyReader,
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

impl From<ckb_transaction_cobuild::error::Error> for Error {
    fn from(err: ckb_transaction_cobuild::error::Error) -> Self {
        match err {
            ckb_transaction_cobuild::error::Error::Sys(e) => e.into(),
            ckb_transaction_cobuild::error::Error::MoleculeEncoding => Error::Encoding,
            ckb_transaction_cobuild::error::Error::WrongSighashAll => Error::WrongSighashAll,
            ckb_transaction_cobuild::error::Error::WrongWitnessLayout => Error::WrongWitnessLayout,
            ckb_transaction_cobuild::error::Error::WrongOtxStart => Error::WrongOtxStart,
            ckb_transaction_cobuild::error::Error::AuthError => Error::AuthError,
            ckb_transaction_cobuild::error::Error::WrongOtx => Error::WrongOtx,
            ckb_transaction_cobuild::error::Error::NoSealFound => Error::NoSealFound,
            ckb_transaction_cobuild::error::Error::ScriptHashAbsent => Error::ScriptHashAbsent,
            ckb_transaction_cobuild::error::Error::WrongCount => Error::WrongCount,
            ckb_transaction_cobuild::error::Error::LazyReader(_) => Error::LazyReader,
        }
    }
}
