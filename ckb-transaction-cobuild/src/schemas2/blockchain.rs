extern crate alloc;
use core::convert::TryInto;
use molecule::lazy_reader::{Cursor, Error, NUMBER_SIZE};
#[derive(Clone)]
pub struct Uint32 {
    pub cursor: Cursor,
}
impl From<Cursor> for Uint32 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Uint32 {
    pub fn len(&self) -> usize {
        4
    }
}
impl Uint32 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl Uint32 {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(4usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Uint64 {
    pub cursor: Cursor,
}
impl From<Cursor> for Uint64 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Uint64 {
    pub fn len(&self) -> usize {
        8
    }
}
impl Uint64 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl Uint64 {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(8usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Uint128 {
    pub cursor: Cursor,
}
impl From<Cursor> for Uint128 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Uint128 {
    pub fn len(&self) -> usize {
        16
    }
}
impl Uint128 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl Uint128 {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(16usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Byte32 {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte32 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte32 {
    pub fn len(&self) -> usize {
        32
    }
}
impl Byte32 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl Byte32 {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(32usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Uint256 {
    pub cursor: Cursor,
}
impl From<Cursor> for Uint256 {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Uint256 {
    pub fn len(&self) -> usize {
        32
    }
}
impl Uint256 {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl Uint256 {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(32usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Bytes {
    pub cursor: Cursor,
}
impl From<Cursor> for Bytes {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Bytes {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl Bytes {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.fixvec_slice_by_index(1usize, index)?;
        cur.try_into()
    }
}
pub struct BytesIterator {
    cur: Bytes,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for BytesIterator {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for Bytes {
    type Item = u8;
    type IntoIter = BytesIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct BytesIteratorRef<'a> {
    cur: &'a Bytes,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for BytesIteratorRef<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl Bytes {
    pub fn iter(&self) -> BytesIteratorRef {
        let len = self.len().unwrap();
        BytesIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl Bytes {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixvec(1usize)?;
        Ok(())
    }
}
pub struct BytesOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for BytesOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
#[derive(Clone)]
pub struct BytesOptVec {
    pub cursor: Cursor,
}
impl From<Cursor> for BytesOptVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl BytesOptVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl BytesOptVec {
    pub fn get(&self, index: usize) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
pub struct BytesOptVecIterator {
    cur: BytesOptVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for BytesOptVecIterator {
    type Item = Option<Cursor>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for BytesOptVec {
    type Item = Option<Cursor>;
    type IntoIter = BytesOptVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct BytesOptVecIteratorRef<'a> {
    cur: &'a BytesOptVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for BytesOptVecIteratorRef<'a> {
    type Item = Option<Cursor>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl BytesOptVec {
    pub fn iter(&self) -> BytesOptVecIteratorRef {
        let len = self.len().unwrap();
        BytesOptVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl BytesOptVec {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct BytesVec {
    pub cursor: Cursor,
}
impl From<Cursor> for BytesVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl BytesVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl BytesVec {
    pub fn get(&self, index: usize) -> Result<Cursor, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        cur.convert_to_rawbytes()
    }
}
pub struct BytesVecIterator {
    cur: BytesVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for BytesVecIterator {
    type Item = Cursor;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for BytesVec {
    type Item = Cursor;
    type IntoIter = BytesVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct BytesVecIteratorRef<'a> {
    cur: &'a BytesVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for BytesVecIteratorRef<'a> {
    type Item = Cursor;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl BytesVec {
    pub fn iter(&self) -> BytesVecIteratorRef {
        let len = self.len().unwrap();
        BytesVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl BytesVec {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Byte32Vec {
    pub cursor: Cursor,
}
impl From<Cursor> for Byte32Vec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Byte32Vec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl Byte32Vec {
    pub fn get(&self, index: usize) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.fixvec_slice_by_index(32usize, index)?;
        cur.try_into()
    }
}
pub struct Byte32VecIterator {
    cur: Byte32Vec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for Byte32VecIterator {
    type Item = [u8; 32usize];
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for Byte32Vec {
    type Item = [u8; 32usize];
    type IntoIter = Byte32VecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct Byte32VecIteratorRef<'a> {
    cur: &'a Byte32Vec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for Byte32VecIteratorRef<'a> {
    type Item = [u8; 32usize];
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl Byte32Vec {
    pub fn iter(&self) -> Byte32VecIteratorRef {
        let len = self.len().unwrap();
        Byte32VecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl Byte32Vec {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixvec(32usize)?;
        Ok(())
    }
}
pub struct ScriptOpt {
    pub cursor: Cursor,
}
impl From<Cursor> for ScriptOpt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
#[derive(Clone)]
pub struct ProposalShortId {
    pub cursor: Cursor,
}
impl From<Cursor> for ProposalShortId {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl ProposalShortId {
    pub fn len(&self) -> usize {
        10
    }
}
impl ProposalShortId {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl ProposalShortId {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(10usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct UncleBlockVec {
    pub cursor: Cursor,
}
impl From<Cursor> for UncleBlockVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl UncleBlockVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl UncleBlockVec {
    pub fn get(&self, index: usize) -> Result<UncleBlock, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        Ok(cur.into())
    }
}
pub struct UncleBlockVecIterator {
    cur: UncleBlockVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for UncleBlockVecIterator {
    type Item = UncleBlock;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for UncleBlockVec {
    type Item = UncleBlock;
    type IntoIter = UncleBlockVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct UncleBlockVecIteratorRef<'a> {
    cur: &'a UncleBlockVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for UncleBlockVecIteratorRef<'a> {
    type Item = UncleBlock;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl UncleBlockVec {
    pub fn iter(&self) -> UncleBlockVecIteratorRef {
        let len = self.len().unwrap();
        UncleBlockVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl UncleBlockVec {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        for i in 0..self.len()? {
            self.get(i)?.verify(compatible)?;
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct TransactionVec {
    pub cursor: Cursor,
}
impl From<Cursor> for TransactionVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl TransactionVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl TransactionVec {
    pub fn get(&self, index: usize) -> Result<Transaction, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        Ok(cur.into())
    }
}
pub struct TransactionVecIterator {
    cur: TransactionVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for TransactionVecIterator {
    type Item = Transaction;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for TransactionVec {
    type Item = Transaction;
    type IntoIter = TransactionVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct TransactionVecIteratorRef<'a> {
    cur: &'a TransactionVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for TransactionVecIteratorRef<'a> {
    type Item = Transaction;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl TransactionVec {
    pub fn iter(&self) -> TransactionVecIteratorRef {
        let len = self.len().unwrap();
        TransactionVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl TransactionVec {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        for i in 0..self.len()? {
            self.get(i)?.verify(compatible)?;
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct ProposalShortIdVec {
    pub cursor: Cursor,
}
impl From<Cursor> for ProposalShortIdVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl ProposalShortIdVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl ProposalShortIdVec {
    pub fn get(&self, index: usize) -> Result<[u8; 10usize], Error> {
        let cur = self.cursor.fixvec_slice_by_index(10usize, index)?;
        cur.try_into()
    }
}
pub struct ProposalShortIdVecIterator {
    cur: ProposalShortIdVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for ProposalShortIdVecIterator {
    type Item = [u8; 10usize];
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for ProposalShortIdVec {
    type Item = [u8; 10usize];
    type IntoIter = ProposalShortIdVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct ProposalShortIdVecIteratorRef<'a> {
    cur: &'a ProposalShortIdVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for ProposalShortIdVecIteratorRef<'a> {
    type Item = [u8; 10usize];
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl ProposalShortIdVec {
    pub fn iter(&self) -> ProposalShortIdVecIteratorRef {
        let len = self.len().unwrap();
        ProposalShortIdVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl ProposalShortIdVec {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixvec(10usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct CellDepVec {
    pub cursor: Cursor,
}
impl From<Cursor> for CellDepVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl CellDepVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl CellDepVec {
    pub fn get(&self, index: usize) -> Result<CellDep, Error> {
        let cur = self.cursor.fixvec_slice_by_index(37usize, index)?;
        Ok(cur.into())
    }
}
pub struct CellDepVecIterator {
    cur: CellDepVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for CellDepVecIterator {
    type Item = CellDep;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for CellDepVec {
    type Item = CellDep;
    type IntoIter = CellDepVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct CellDepVecIteratorRef<'a> {
    cur: &'a CellDepVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for CellDepVecIteratorRef<'a> {
    type Item = CellDep;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl CellDepVec {
    pub fn iter(&self) -> CellDepVecIteratorRef {
        let len = self.len().unwrap();
        CellDepVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl CellDepVec {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixvec(37usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct CellInputVec {
    pub cursor: Cursor,
}
impl From<Cursor> for CellInputVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl CellInputVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl CellInputVec {
    pub fn get(&self, index: usize) -> Result<CellInput, Error> {
        let cur = self.cursor.fixvec_slice_by_index(44usize, index)?;
        Ok(cur.into())
    }
}
pub struct CellInputVecIterator {
    cur: CellInputVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for CellInputVecIterator {
    type Item = CellInput;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for CellInputVec {
    type Item = CellInput;
    type IntoIter = CellInputVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct CellInputVecIteratorRef<'a> {
    cur: &'a CellInputVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for CellInputVecIteratorRef<'a> {
    type Item = CellInput;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl CellInputVec {
    pub fn iter(&self) -> CellInputVecIteratorRef {
        let len = self.len().unwrap();
        CellInputVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl CellInputVec {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixvec(44usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct CellOutputVec {
    pub cursor: Cursor,
}
impl From<Cursor> for CellOutputVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl CellOutputVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl CellOutputVec {
    pub fn get(&self, index: usize) -> Result<CellOutput, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        Ok(cur.into())
    }
}
pub struct CellOutputVecIterator {
    cur: CellOutputVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for CellOutputVecIterator {
    type Item = CellOutput;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl core::iter::IntoIterator for CellOutputVec {
    type Item = CellOutput;
    type IntoIter = CellOutputVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct CellOutputVecIteratorRef<'a> {
    cur: &'a CellOutputVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for CellOutputVecIteratorRef<'a> {
    type Item = CellOutput;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let res = self.cur.get(self.index).unwrap();
            self.index += 1;
            Some(res)
        }
    }
}
impl CellOutputVec {
    pub fn iter(&self) -> CellOutputVecIteratorRef {
        let len = self.len().unwrap();
        CellOutputVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl CellOutputVec {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        for i in 0..self.len()? {
            self.get(i)?.verify(compatible)?;
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct Script {
    pub cursor: Cursor,
}
impl From<Cursor> for Script {
    fn from(cursor: Cursor) -> Self {
        Script { cursor }
    }
}
impl Script {
    pub fn code_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.try_into()
    }
}
impl Script {
    pub fn hash_type(&self) -> Result<u8, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.try_into()
    }
}
impl Script {
    pub fn args(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        cur.convert_to_rawbytes()
    }
}
impl Script {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(3usize, compatible)?;
        Byte32::from(Cursor::try_from(self.code_hash()?)?).verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct OutPoint {
    pub cursor: Cursor,
}
impl From<Cursor> for OutPoint {
    fn from(cursor: Cursor) -> Self {
        OutPoint { cursor }
    }
}
impl OutPoint {
    pub fn tx_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.slice_by_offset(0usize, 32usize)?;
        cur.try_into()
    }
}
impl OutPoint {
    pub fn index(&self) -> Result<u32, Error> {
        let cur = self.cursor.slice_by_offset(32usize, 4usize)?;
        cur.try_into()
    }
}
impl OutPoint {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(36usize)?;
        Byte32::from(Cursor::try_from(self.tx_hash()?)?).verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct CellInput {
    pub cursor: Cursor,
}
impl From<Cursor> for CellInput {
    fn from(cursor: Cursor) -> Self {
        CellInput { cursor }
    }
}
impl CellInput {
    pub fn since(&self) -> Result<u64, Error> {
        let cur = self.cursor.slice_by_offset(0usize, 8usize)?;
        cur.try_into()
    }
}
impl CellInput {
    pub fn previous_output(&self) -> Result<OutPoint, Error> {
        let cur = self.cursor.slice_by_offset(8usize, 36usize)?;
        Ok(cur.into())
    }
}
impl CellInput {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(44usize)?;
        self.previous_output()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct CellOutput {
    pub cursor: Cursor,
}
impl From<Cursor> for CellOutput {
    fn from(cursor: Cursor) -> Self {
        CellOutput { cursor }
    }
}
impl CellOutput {
    pub fn capacity(&self) -> Result<u64, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.try_into()
    }
}
impl CellOutput {
    pub fn lock(&self) -> Result<Script, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl CellOutput {
    pub fn type_(&self) -> Result<Option<Script>, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.try_into()?))
        }
    }
}
impl CellOutput {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(3usize, compatible)?;
        self.lock()?.verify(compatible)?;
        let val = self.type_()?;
        if val.is_some() {
            let val = val.unwrap();
            val.verify(compatible)?;
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct CellDep {
    pub cursor: Cursor,
}
impl From<Cursor> for CellDep {
    fn from(cursor: Cursor) -> Self {
        CellDep { cursor }
    }
}
impl CellDep {
    pub fn out_point(&self) -> Result<OutPoint, Error> {
        let cur = self.cursor.slice_by_offset(0usize, 36usize)?;
        Ok(cur.into())
    }
}
impl CellDep {
    pub fn dep_type(&self) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(36usize, 1usize)?;
        cur.try_into()
    }
}
impl CellDep {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(37usize)?;
        self.out_point()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct RawTransaction {
    pub cursor: Cursor,
}
impl From<Cursor> for RawTransaction {
    fn from(cursor: Cursor) -> Self {
        RawTransaction { cursor }
    }
}
impl RawTransaction {
    pub fn version(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.try_into()
    }
}
impl RawTransaction {
    pub fn cell_deps(&self) -> Result<CellDepVec, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl RawTransaction {
    pub fn header_deps(&self) -> Result<Byte32Vec, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        Ok(cur.into())
    }
}
impl RawTransaction {
    pub fn inputs(&self) -> Result<CellInputVec, Error> {
        let cur = self.cursor.table_slice_by_index(3usize)?;
        Ok(cur.into())
    }
}
impl RawTransaction {
    pub fn outputs(&self) -> Result<CellOutputVec, Error> {
        let cur = self.cursor.table_slice_by_index(4usize)?;
        Ok(cur.into())
    }
}
impl RawTransaction {
    pub fn outputs_data(&self) -> Result<BytesVec, Error> {
        let cur = self.cursor.table_slice_by_index(5usize)?;
        Ok(cur.into())
    }
}
impl RawTransaction {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(6usize, compatible)?;
        self.outputs()?.verify(compatible)?;
        self.outputs_data()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Transaction {
    pub cursor: Cursor,
}
impl From<Cursor> for Transaction {
    fn from(cursor: Cursor) -> Self {
        Transaction { cursor }
    }
}
impl Transaction {
    pub fn raw(&self) -> Result<RawTransaction, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl Transaction {
    pub fn witnesses(&self) -> Result<BytesVec, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl Transaction {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(2usize, compatible)?;
        self.raw()?.verify(compatible)?;
        self.witnesses()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct RawHeader {
    pub cursor: Cursor,
}
impl From<Cursor> for RawHeader {
    fn from(cursor: Cursor) -> Self {
        RawHeader { cursor }
    }
}
impl RawHeader {
    pub fn version(&self) -> Result<u32, Error> {
        let cur = self.cursor.slice_by_offset(0usize, 4usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn compact_target(&self) -> Result<u32, Error> {
        let cur = self.cursor.slice_by_offset(4usize, 4usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn timestamp(&self) -> Result<u64, Error> {
        let cur = self.cursor.slice_by_offset(8usize, 8usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn number(&self) -> Result<u64, Error> {
        let cur = self.cursor.slice_by_offset(16usize, 8usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn epoch(&self) -> Result<u64, Error> {
        let cur = self.cursor.slice_by_offset(24usize, 8usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn parent_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.slice_by_offset(32usize, 32usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn transactions_root(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.slice_by_offset(64usize, 32usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn proposals_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.slice_by_offset(96usize, 32usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn extra_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.slice_by_offset(128usize, 32usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn dao(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.slice_by_offset(160usize, 32usize)?;
        cur.try_into()
    }
}
impl RawHeader {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(192usize)?;
        Byte32::from(Cursor::try_from(self.parent_hash()?)?).verify(compatible)?;
        Byte32::from(Cursor::try_from(self.transactions_root()?)?).verify(compatible)?;
        Byte32::from(Cursor::try_from(self.proposals_hash()?)?).verify(compatible)?;
        Byte32::from(Cursor::try_from(self.extra_hash()?)?).verify(compatible)?;
        Byte32::from(Cursor::try_from(self.dao()?)?).verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Header {
    pub cursor: Cursor,
}
impl From<Cursor> for Header {
    fn from(cursor: Cursor) -> Self {
        Header { cursor }
    }
}
impl Header {
    pub fn raw(&self) -> Result<RawHeader, Error> {
        let cur = self.cursor.slice_by_offset(0usize, 192usize)?;
        Ok(cur.into())
    }
}
impl Header {
    pub fn nonce(&self) -> Result<[u8; 16usize], Error> {
        let cur = self.cursor.slice_by_offset(192usize, 16usize)?;
        cur.try_into()
    }
}
impl Header {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(208usize)?;
        self.raw()?.verify(compatible)?;
        Uint128::from(Cursor::try_from(self.nonce()?)?).verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct UncleBlock {
    pub cursor: Cursor,
}
impl From<Cursor> for UncleBlock {
    fn from(cursor: Cursor) -> Self {
        UncleBlock { cursor }
    }
}
impl UncleBlock {
    pub fn header(&self) -> Result<Header, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl UncleBlock {
    pub fn proposals(&self) -> Result<ProposalShortIdVec, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl UncleBlock {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(2usize, compatible)?;
        self.header()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Block {
    pub cursor: Cursor,
}
impl From<Cursor> for Block {
    fn from(cursor: Cursor) -> Self {
        Block { cursor }
    }
}
impl Block {
    pub fn header(&self) -> Result<Header, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl Block {
    pub fn uncles(&self) -> Result<UncleBlockVec, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl Block {
    pub fn transactions(&self) -> Result<TransactionVec, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        Ok(cur.into())
    }
}
impl Block {
    pub fn proposals(&self) -> Result<ProposalShortIdVec, Error> {
        let cur = self.cursor.table_slice_by_index(3usize)?;
        Ok(cur.into())
    }
}
impl Block {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(4usize, compatible)?;
        self.header()?.verify(compatible)?;
        self.uncles()?.verify(compatible)?;
        self.transactions()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct BlockV1 {
    pub cursor: Cursor,
}
impl From<Cursor> for BlockV1 {
    fn from(cursor: Cursor) -> Self {
        BlockV1 { cursor }
    }
}
impl BlockV1 {
    pub fn header(&self) -> Result<Header, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl BlockV1 {
    pub fn uncles(&self) -> Result<UncleBlockVec, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl BlockV1 {
    pub fn transactions(&self) -> Result<TransactionVec, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        Ok(cur.into())
    }
}
impl BlockV1 {
    pub fn proposals(&self) -> Result<ProposalShortIdVec, Error> {
        let cur = self.cursor.table_slice_by_index(3usize)?;
        Ok(cur.into())
    }
}
impl BlockV1 {
    pub fn extension(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(4usize)?;
        cur.convert_to_rawbytes()
    }
}
impl BlockV1 {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(5usize, compatible)?;
        self.header()?.verify(compatible)?;
        self.uncles()?.verify(compatible)?;
        self.transactions()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct CellbaseWitness {
    pub cursor: Cursor,
}
impl From<Cursor> for CellbaseWitness {
    fn from(cursor: Cursor) -> Self {
        CellbaseWitness { cursor }
    }
}
impl CellbaseWitness {
    pub fn lock(&self) -> Result<Script, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl CellbaseWitness {
    pub fn message(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.convert_to_rawbytes()
    }
}
impl CellbaseWitness {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(2usize, compatible)?;
        self.lock()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct WitnessArgs {
    pub cursor: Cursor,
}
impl From<Cursor> for WitnessArgs {
    fn from(cursor: Cursor) -> Self {
        WitnessArgs { cursor }
    }
}
impl WitnessArgs {
    pub fn lock(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
impl WitnessArgs {
    pub fn input_type(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
impl WitnessArgs {
    pub fn output_type(&self) -> Result<Option<Cursor>, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            let cur = cur.convert_to_rawbytes()?;
            Ok(Some(cur.into()))
        }
    }
}
impl WitnessArgs {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(3usize, compatible)?;
        Ok(())
    }
}
