use ckb_std::error::SysError;
use ckb_typed_message;

/// Error
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    AuthError,
    WrongSighashWithAction,
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

impl From<ckb_typed_message::Error> for Error {
    fn from(err: ckb_typed_message::Error) -> Self {
        match err {
            ckb_typed_message::Error::Sys(e) => e.into(),
            ckb_typed_message::Error::DuplicateAction => Error::WrongSighashWithAction,
            ckb_typed_message::Error::MoleculeEncoding => Error::Encoding,
            ckb_typed_message::Error::WrongSighashWithAction => Error::WrongSighashWithAction,
        }
    }
}
