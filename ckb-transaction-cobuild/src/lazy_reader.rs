use core::cmp::min;

use super::schemas2::blockchain;
use alloc::boxed::Box;
use ckb_std::{ckb_constants::Source, error::SysError, syscalls};

pub use molecule::lazy_reader::{Cursor, Error, Read};

fn read_data<F: Fn(&mut [u8], usize) -> Result<usize, SysError>>(
    load_func: F,
    buf: &mut [u8],
    offset: usize,
    total_size: usize,
) -> Result<usize, Error> {
    if offset >= total_size {
        return Err(Error::OutOfBound(offset, total_size));
    }
    let remaining_len = total_size - offset;
    let min_len = min(remaining_len, buf.len());

    if (offset + min_len) > total_size {
        return Err(Error::OutOfBound(offset + min_len, total_size));
    }
    let actual_len = match load_func(buf, offset) {
        Ok(l) => l,
        Err(err) => match err {
            SysError::LengthNotEnough(l) => l,
            _ => return Err(Error::OutOfBound(0, 0)),
        },
    };
    let read_len = min(buf.len(), actual_len);
    Ok(read_len)
}

fn read_size<F: Fn(&mut [u8]) -> Result<usize, SysError>>(load_func: F) -> Result<usize, Error> {
    let mut buf = [0u8; 4];
    match load_func(&mut buf) {
        Ok(l) => Ok(l),
        Err(e) => match e {
            SysError::LengthNotEnough(l) => Ok(l),
            _ => Err(Error::OutOfBound(0, 0)),
        },
    }
}

pub struct TransactionReader {
    pub total_size: usize,
}

impl TransactionReader {
    pub fn new() -> Self {
        let total_size = read_size(|buf| syscalls::load_transaction(buf, 0)).unwrap();
        Self { total_size }
    }
}

impl Read for TransactionReader {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        read_data(
            |buf, offset| syscalls::load_transaction(buf, offset),
            buf,
            offset,
            self.total_size,
        )
    }
}

impl From<TransactionReader> for Cursor {
    fn from(data: TransactionReader) -> Self {
        Cursor::new(data.total_size, Box::new(data))
    }
}

pub fn new_transaction() -> blockchain::Transaction {
    let tx_reader = TransactionReader::new();
    let cursor: Cursor = tx_reader.into();
    blockchain::Transaction::from(cursor)
}

// Input cell are not in current transaction. Can't use `TransactionReader`
pub struct InputCellReader {
    pub total_size: usize,
    pub index: usize,
    pub source: Source,
}

impl InputCellReader {
    pub fn try_new(index: usize, source: Source) -> Result<Self, Error> {
        let total_size = read_size(|buf| syscalls::load_cell(buf, 0, index, source))?;
        Ok(Self {
            total_size,
            index,
            source,
        })
    }
}

impl Read for InputCellReader {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        read_data(
            |buf, offset| syscalls::load_cell(buf, offset, self.index, self.source),
            buf,
            offset,
            self.total_size,
        )
    }
}

impl From<InputCellReader> for Cursor {
    fn from(data: InputCellReader) -> Self {
        Cursor::new(data.total_size, Box::new(data))
    }
}

pub fn new_input_cell(index: usize, source: Source) -> Result<blockchain::CellOutput, Error> {
    let reader = InputCellReader::try_new(index, source)?;
    let cursor: Cursor = reader.into();
    Ok(blockchain::CellOutput::from(cursor))
}

// Input cell data are not in current transaction. Can't use `TransactionReader`
pub struct InputCellDataReader {
    pub total_size: usize,
    pub index: usize,
    pub source: Source,
}

impl InputCellDataReader {
    pub fn try_new(index: usize, source: Source) -> Result<Self, Error> {
        let total_size = read_size(|buf| syscalls::load_cell_data(buf, 0, index, source))?;
        Ok(Self {
            total_size,
            index,
            source,
        })
    }
}

impl Read for InputCellDataReader {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        read_data(
            |buf, offset| syscalls::load_cell_data(buf, offset, self.index, self.source),
            buf,
            offset,
            self.total_size,
        )
    }
}

impl From<InputCellDataReader> for Cursor {
    fn from(data: InputCellDataReader) -> Self {
        Cursor::new(data.total_size, Box::new(data))
    }
}

pub fn new_input_cell_data(index: usize, source: Source) -> Result<Cursor, Error> {
    let reader = InputCellDataReader::try_new(index, source)?;
    let cursor: Cursor = reader.into();
    Ok(cursor)
}

pub struct WitnessReader {
    pub total_size: usize,
    pub index: usize,
    pub source: Source,
}

impl WitnessReader {
    pub fn try_new(index: usize, source: Source) -> Result<Self, Error> {
        let total_size = read_size(|buf| syscalls::load_witness(buf, 0, index, source))?;
        Ok(Self {
            total_size,
            index,
            source,
        })
    }
}

impl Read for WitnessReader {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        read_data(
            |buf, offset| syscalls::load_witness(buf, offset, self.index, self.source),
            buf,
            offset,
            self.total_size,
        )
    }
}

impl From<WitnessReader> for Cursor {
    fn from(data: WitnessReader) -> Self {
        Cursor::new(data.total_size, Box::new(data))
    }
}

pub fn new_witness(index: usize, source: Source) -> Result<Cursor, Error> {
    let reader = WitnessReader::try_new(index, source)?;
    let cursor: Cursor = reader.into();
    Ok(cursor)
}
