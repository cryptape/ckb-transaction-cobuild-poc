extern crate alloc;
use super::blockchain::*;
use core::convert::TryInto;
use molecule::lazy_reader::{Cursor, Error, NUMBER_SIZE};
#[derive(Clone)]
pub struct Hash {
    pub cursor: Cursor,
}
impl From<Cursor> for Hash {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl Hash {
    pub fn len(&self) -> usize {
        32
    }
}
impl Hash {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.slice_by_offset(1usize * index, 1usize)?;
        cur.try_into()
    }
}
impl Hash {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixed_size(32usize)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct String {
    pub cursor: Cursor,
}
impl From<Cursor> for String {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl String {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.fixvec_length()
    }
}
impl String {
    pub fn get(&self, index: usize) -> Result<u8, Error> {
        let cur = self.cursor.fixvec_slice_by_index(1usize, index)?;
        cur.try_into()
    }
}
pub struct StringIterator {
    cur: String,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for StringIterator {
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
impl core::iter::IntoIterator for String {
    type Item = u8;
    type IntoIter = StringIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct StringIteratorRef<'a> {
    cur: &'a String,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for StringIteratorRef<'a> {
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
impl String {
    pub fn iter(&self) -> StringIteratorRef {
        let len = self.len().unwrap();
        StringIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl String {
    pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
        self.cursor.verify_fixvec(1usize)?;
        Ok(())
    }
}
pub struct Uint32Opt {
    pub cursor: Cursor,
}
impl From<Cursor> for Uint32Opt {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
#[derive(Clone)]
pub struct Action {
    pub cursor: Cursor,
}
impl From<Cursor> for Action {
    fn from(cursor: Cursor) -> Self {
        Action { cursor }
    }
}
impl Action {
    pub fn script_info_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.try_into()
    }
}
impl Action {
    pub fn script_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.try_into()
    }
}
impl Action {
    pub fn data(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        cur.convert_to_rawbytes()
    }
}
impl Action {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(3usize, compatible)?;
        Byte32::from(Cursor::try_from(self.script_info_hash()?)?).verify(compatible)?;
        Byte32::from(Cursor::try_from(self.script_hash()?)?).verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct ActionVec {
    pub cursor: Cursor,
}
impl From<Cursor> for ActionVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl ActionVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl ActionVec {
    pub fn get(&self, index: usize) -> Result<Action, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        Ok(cur.into())
    }
}
pub struct ActionVecIterator {
    cur: ActionVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for ActionVecIterator {
    type Item = Action;
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
impl core::iter::IntoIterator for ActionVec {
    type Item = Action;
    type IntoIter = ActionVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct ActionVecIteratorRef<'a> {
    cur: &'a ActionVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for ActionVecIteratorRef<'a> {
    type Item = Action;
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
impl ActionVec {
    pub fn iter(&self) -> ActionVecIteratorRef {
        let len = self.len().unwrap();
        ActionVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl ActionVec {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        for i in 0..self.len()? {
            self.get(i)?.verify(compatible)?;
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct Message {
    pub cursor: Cursor,
}
impl From<Cursor> for Message {
    fn from(cursor: Cursor) -> Self {
        Message { cursor }
    }
}
impl Message {
    pub fn actions(&self) -> Result<ActionVec, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl Message {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(1usize, compatible)?;
        self.actions()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct ScriptInfo {
    pub cursor: Cursor,
}
impl From<Cursor> for ScriptInfo {
    fn from(cursor: Cursor) -> Self {
        ScriptInfo { cursor }
    }
}
impl ScriptInfo {
    pub fn name(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.convert_to_rawbytes()
    }
}
impl ScriptInfo {
    pub fn url(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.convert_to_rawbytes()
    }
}
impl ScriptInfo {
    pub fn script_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        cur.try_into()
    }
}
impl ScriptInfo {
    pub fn schema(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(3usize)?;
        cur.convert_to_rawbytes()
    }
}
impl ScriptInfo {
    pub fn message_type(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(4usize)?;
        cur.convert_to_rawbytes()
    }
}
impl ScriptInfo {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(5usize, compatible)?;
        Byte32::from(Cursor::try_from(self.script_hash()?)?).verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct ScriptInfoVec {
    pub cursor: Cursor,
}
impl From<Cursor> for ScriptInfoVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl ScriptInfoVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl ScriptInfoVec {
    pub fn get(&self, index: usize) -> Result<ScriptInfo, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        Ok(cur.into())
    }
}
pub struct ScriptInfoVecIterator {
    cur: ScriptInfoVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for ScriptInfoVecIterator {
    type Item = ScriptInfo;
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
impl core::iter::IntoIterator for ScriptInfoVec {
    type Item = ScriptInfo;
    type IntoIter = ScriptInfoVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct ScriptInfoVecIteratorRef<'a> {
    cur: &'a ScriptInfoVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for ScriptInfoVecIteratorRef<'a> {
    type Item = ScriptInfo;
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
impl ScriptInfoVec {
    pub fn iter(&self) -> ScriptInfoVecIteratorRef {
        let len = self.len().unwrap();
        ScriptInfoVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl ScriptInfoVec {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        for i in 0..self.len()? {
            self.get(i)?.verify(compatible)?;
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct ResolvedInputs {
    pub cursor: Cursor,
}
impl From<Cursor> for ResolvedInputs {
    fn from(cursor: Cursor) -> Self {
        ResolvedInputs { cursor }
    }
}
impl ResolvedInputs {
    pub fn outputs(&self) -> Result<CellOutputVec, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl ResolvedInputs {
    pub fn outputs_data(&self) -> Result<BytesVec, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl ResolvedInputs {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(2usize, compatible)?;
        self.outputs()?.verify(compatible)?;
        self.outputs_data()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct BuildingPacketV1 {
    pub cursor: Cursor,
}
impl From<Cursor> for BuildingPacketV1 {
    fn from(cursor: Cursor) -> Self {
        BuildingPacketV1 { cursor }
    }
}
impl BuildingPacketV1 {
    pub fn message(&self) -> Result<Message, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl BuildingPacketV1 {
    pub fn payload(&self) -> Result<Transaction, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        Ok(cur.into())
    }
}
impl BuildingPacketV1 {
    pub fn resolved_inputs(&self) -> Result<ResolvedInputs, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        Ok(cur.into())
    }
}
impl BuildingPacketV1 {
    pub fn change_output(&self) -> Result<Option<u32>, Error> {
        let cur = self.cursor.table_slice_by_index(3usize)?;
        if cur.option_is_none() {
            Ok(None)
        } else {
            Ok(Some(cur.try_into()?))
        }
    }
}
impl BuildingPacketV1 {
    pub fn script_infos(&self) -> Result<ScriptInfoVec, Error> {
        let cur = self.cursor.table_slice_by_index(4usize)?;
        Ok(cur.into())
    }
}
impl BuildingPacketV1 {
    pub fn lock_actions(&self) -> Result<ActionVec, Error> {
        let cur = self.cursor.table_slice_by_index(5usize)?;
        Ok(cur.into())
    }
}
impl BuildingPacketV1 {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(6usize, compatible)?;
        self.message()?.verify(compatible)?;
        self.payload()?.verify(compatible)?;
        self.resolved_inputs()?.verify(compatible)?;
        self.script_infos()?.verify(compatible)?;
        self.lock_actions()?.verify(compatible)?;
        Ok(())
    }
}
pub enum BuildingPacket {
    BuildingPacketV1(BuildingPacketV1),
}
impl TryFrom<Cursor> for BuildingPacket {
    type Error = Error;
    fn try_from(cur: Cursor) -> Result<Self, Self::Error> {
        let item = cur.union_unpack()?;
        let mut cur = cur;
        cur.add_offset(NUMBER_SIZE)?;
        cur.sub_size(NUMBER_SIZE)?;
        match item.item_id {
            0usize => Ok(Self::BuildingPacketV1(cur.into())),
            _ => Err(Error::UnknownItem),
        }
    }
}
impl BuildingPacket {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        match self {
            Self::BuildingPacketV1(v) => {
                v.verify(compatible)?;
                Ok(())
            }
        }
    }
}
#[derive(Clone)]
pub struct SighashAll {
    pub cursor: Cursor,
}
impl From<Cursor> for SighashAll {
    fn from(cursor: Cursor) -> Self {
        SighashAll { cursor }
    }
}
impl SighashAll {
    pub fn message(&self) -> Result<Message, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        Ok(cur.into())
    }
}
impl SighashAll {
    pub fn seal(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.convert_to_rawbytes()
    }
}
impl SighashAll {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(2usize, compatible)?;
        self.message()?.verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct SighashAllOnly {
    pub cursor: Cursor,
}
impl From<Cursor> for SighashAllOnly {
    fn from(cursor: Cursor) -> Self {
        SighashAllOnly { cursor }
    }
}
impl SighashAllOnly {
    pub fn seal(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.convert_to_rawbytes()
    }
}
impl SighashAllOnly {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(1usize, compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct SealPair {
    pub cursor: Cursor,
}
impl From<Cursor> for SealPair {
    fn from(cursor: Cursor) -> Self {
        SealPair { cursor }
    }
}
impl SealPair {
    pub fn script_hash(&self) -> Result<[u8; 32usize], Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.try_into()
    }
}
impl SealPair {
    pub fn seal(&self) -> Result<Cursor, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.convert_to_rawbytes()
    }
}
impl SealPair {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(2usize, compatible)?;
        Byte32::from(Cursor::try_from(self.script_hash()?)?).verify(compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct SealPairVec {
    pub cursor: Cursor,
}
impl From<Cursor> for SealPairVec {
    fn from(cursor: Cursor) -> Self {
        Self { cursor }
    }
}
impl SealPairVec {
    pub fn len(&self) -> Result<usize, Error> {
        self.cursor.dynvec_length()
    }
}
impl SealPairVec {
    pub fn get(&self, index: usize) -> Result<SealPair, Error> {
        let cur = self.cursor.dynvec_slice_by_index(index)?;
        Ok(cur.into())
    }
}
pub struct SealPairVecIterator {
    cur: SealPairVec,
    index: usize,
    len: usize,
}
impl core::iter::Iterator for SealPairVecIterator {
    type Item = SealPair;
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
impl core::iter::IntoIterator for SealPairVec {
    type Item = SealPair;
    type IntoIter = SealPairVecIterator;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len().unwrap();
        Self::IntoIter {
            cur: self,
            index: 0,
            len,
        }
    }
}
pub struct SealPairVecIteratorRef<'a> {
    cur: &'a SealPairVec,
    index: usize,
    len: usize,
}
impl<'a> core::iter::Iterator for SealPairVecIteratorRef<'a> {
    type Item = SealPair;
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
impl SealPairVec {
    pub fn iter(&self) -> SealPairVecIteratorRef {
        let len = self.len().unwrap();
        SealPairVecIteratorRef {
            cur: &self,
            index: 0,
            len,
        }
    }
}
impl SealPairVec {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_dynvec()?;
        for i in 0..self.len()? {
            self.get(i)?.verify(compatible)?;
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct OtxStart {
    pub cursor: Cursor,
}
impl From<Cursor> for OtxStart {
    fn from(cursor: Cursor) -> Self {
        OtxStart { cursor }
    }
}
impl OtxStart {
    pub fn start_input_cell(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.try_into()
    }
}
impl OtxStart {
    pub fn start_output_cell(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.try_into()
    }
}
impl OtxStart {
    pub fn start_cell_deps(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        cur.try_into()
    }
}
impl OtxStart {
    pub fn start_header_deps(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(3usize)?;
        cur.try_into()
    }
}
impl OtxStart {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(4usize, compatible)?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct Otx {
    pub cursor: Cursor,
}
impl From<Cursor> for Otx {
    fn from(cursor: Cursor) -> Self {
        Otx { cursor }
    }
}
impl Otx {
    pub fn input_cells(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(0usize)?;
        cur.try_into()
    }
}
impl Otx {
    pub fn output_cells(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(1usize)?;
        cur.try_into()
    }
}
impl Otx {
    pub fn cell_deps(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(2usize)?;
        cur.try_into()
    }
}
impl Otx {
    pub fn header_deps(&self) -> Result<u32, Error> {
        let cur = self.cursor.table_slice_by_index(3usize)?;
        cur.try_into()
    }
}
impl Otx {
    pub fn message(&self) -> Result<Message, Error> {
        let cur = self.cursor.table_slice_by_index(4usize)?;
        Ok(cur.into())
    }
}
impl Otx {
    pub fn seals(&self) -> Result<SealPairVec, Error> {
        let cur = self.cursor.table_slice_by_index(5usize)?;
        Ok(cur.into())
    }
}
impl Otx {
    pub fn verify(&self, compatible: bool) -> Result<(), Error> {
        self.cursor.verify_table(6usize, compatible)?;
        self.message()?.verify(compatible)?;
        self.seals()?.verify(compatible)?;
        Ok(())
    }
}
