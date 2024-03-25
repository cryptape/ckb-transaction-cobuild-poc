extern crate alloc;
use super::basic::*;
use core::convert::TryInto;
use molecule::lazy_reader::{Cursor, Error, NUMBER_SIZE};
pub enum WitnessLayout {
    SighashAll(SighashAll),
    SighashAllOnly(SighashAllOnly),
    Otx(Otx),
    OtxStart(OtxStart),
}
impl TryFrom<Cursor> for WitnessLayout {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Self::Error> {
        let item = cur.union_unpack()?;
        let mut cur = cur;
        cur.add_offset(NUMBER_SIZE)?;
        cur.sub_size(NUMBER_SIZE)?;
        match item.item_id {
            4278190081usize => Ok(Self::SighashAll(cur.into())),
            4278190082usize => Ok(Self::SighashAllOnly(cur.into())),
            4278190083usize => Ok(Self::Otx(cur.into())),
            4278190084usize => Ok(Self::OtxStart(cur.into())),
            _ => Err(Error::UnknownItem),
        }
    }
}
impl WitnessLayout {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        match self {
            Self::SighashAll(v) => {
                v.verify(compatible)?;
                Ok(())
            }
            Self::SighashAllOnly(v) => {
                v.verify(compatible)?;
                Ok(())
            }
            Self::Otx(v) => {
                v.verify(compatible)?;
                Ok(())
            }
            Self::OtxStart(v) => {
                v.verify(compatible)?;
                Ok(())
            }
        }
    }
}
