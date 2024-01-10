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

impl From<ckb_transaction_cobuild::Error> for Error {
    fn from(err: ckb_transaction_cobuild::Error) -> Self {
        match err {
            ckb_transaction_cobuild::Error::Sys(e) => e.into(),
            ckb_transaction_cobuild::Error::MoleculeEncoding => Error::Encoding,
            ckb_transaction_cobuild::Error::WrongSighashAll => Error::WrongSighashAll,
            ckb_transaction_cobuild::Error::WrongWitnessLayout => Error::WrongWitnessLayout,
            ckb_transaction_cobuild::Error::WrongOtxStart => Error::WrongOtxStart,
        }
    }
}
