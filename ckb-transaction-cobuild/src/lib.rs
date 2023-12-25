#![no_std]
extern crate alloc;
pub mod blake2b;
pub mod schemas;

use alloc::vec::Vec;
use blake2b::new_sighash_all_blake2b;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed::CellInput,
    error::SysError,
    high_level::{load_cell, load_cell_data, load_tx_hash, load_witness, QueryIter},
    syscalls::load_transaction,
};
use core::convert::Into;
use molecule::{
    error::VerificationError,
    prelude::{Entity, Reader},
    NUMBER_SIZE,
};
use schemas::{
    basic::SighashAll,
    top_level::{WitnessLayout, WitnessLayoutReader, WitnessLayoutUnion, WitnessLayoutUnionReader},
};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Error {
    Sys(SysError),
    MoleculeEncoding,
    WrongSighashAll,
    WrongWitnessLayout,
}

impl From<SysError> for Error {
    fn from(e: SysError) -> Self {
        Error::Sys(e)
    }
}

impl From<VerificationError> for Error {
    fn from(_: VerificationError) -> Self {
        Error::MoleculeEncoding
    }
}

///
/// Fetch the corresponding WitnessLayout(SighashAll or SighashAllOnly)
/// Used by lock script
///
pub fn fetch_witness_layout() -> Result<WitnessLayout, Error> {
    match load_witness(0, Source::GroupInput) {
        Ok(witness) => {
            if let Ok(r) = WitnessLayoutReader::from_slice(&witness) {
                match r.to_enum() {
                    WitnessLayoutUnionReader::SighashAll(_)
                    | WitnessLayoutUnionReader::SighashAllOnly(_) => Ok(r.to_entity()),
                    _ => Err(Error::MoleculeEncoding),
                }
            } else {
                Err(Error::MoleculeEncoding)
            }
        }
        Err(e) => Err(e.into()),
    }
}

///
/// fetch the only SighashAll from all witnesses.
/// This function can also check the count of SighashAll is one.
///
pub fn fetch_sighash_all() -> Result<SighashAll, Error> {
    let mut iter = QueryIter::new(load_witness, Source::Input).filter_map(|witness| {
        WitnessLayoutReader::from_slice(&witness)
            .ok()
            .and_then(|r| match r.to_enum() {
                WitnessLayoutUnionReader::SighashAll(s) => Some(s.to_entity()),
                _ => None,
            })
    });

    match (iter.next(), iter.next()) {
        (Some(sighash_with_action), None) => Ok(sighash_with_action),
        _ => Err(Error::WrongSighashAll),
    }
}

///
/// for lock script with message, the other witness in script group except
/// first one should be empty
///
pub fn check_others_in_group() -> Result<(), Error> {
    if QueryIter::new(load_witness, Source::GroupInput)
        .skip(1)
        .all(|witness| witness.is_empty())
    {
        Ok(())
    } else {
        Err(Error::WrongWitnessLayout)
    }
}

pub fn generate_signing_message_hash(message: &[u8]) -> Result<[u8; 32], Error> {
    let mut hasher = new_sighash_all_blake2b();
    // message
    hasher.update(&(message.len() as u32).to_le_bytes());
    hasher.update(message);
    // tx hash
    hasher.update(&load_tx_hash()?);
    // inputs cell and data
    let inputs_len = calculate_inputs_len()?;
    for i in 0..inputs_len {
        let input_cell = load_cell(i, Source::Input)?;
        hasher.update(input_cell.as_slice());
        // TODO cell data may be too large, use high_level::load_data fn to load and hash it in chunks
        let input_cell_data = load_cell_data(i, Source::Input)?;
        hasher.update(&(input_cell_data.len() as u32).to_le_bytes());
        hasher.update(&input_cell_data);
    }
    // extra witnesses
    for witness in QueryIter::new(load_witness, Source::Input).skip(inputs_len) {
        hasher.update(&(witness.len() as u32).to_le_bytes());
        hasher.update(&witness);
    }

    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    return Ok(result);
}

///
/// the molecule data structure of transaction is:
/// full-size|raw-offset|witnesses-offset|raw-full-size|version-offset|cell_deps-offset|header_deps-offset|inputs-offset|outputs-offset|...
/// full-size and offset are 4 bytes, so we can read the inputs-offset and outputs-offset at [28, 36),
/// then we can get the length of inputs by calculating the difference between inputs-offset and outputs-offset
///
fn calculate_inputs_len() -> Result<usize, SysError> {
    let mut offsets = [0u8; 8];
    match load_transaction(&mut offsets, 28) {
        // this syscall will always return SysError::LengthNotEnough since we only load 8 bytes, let's ignore it
        Err(SysError::LengthNotEnough(_)) => {}
        Err(SysError::Unknown(e)) => return Err(SysError::Unknown(e)),
        _ => unreachable!(),
    }
    let inputs_offset = u32::from_le_bytes(offsets[0..4].try_into().unwrap());
    let outputs_offset = u32::from_le_bytes(offsets[4..8].try_into().unwrap());
    Ok((outputs_offset as usize - inputs_offset as usize - NUMBER_SIZE) / CellInput::TOTAL_SIZE)
}

///
/// parse transaction with message and return 2 values:
/// 1. signing_message_hash, 32 bytes message for signature verification
/// 2. seal, seal field in SighashAll or SighashAllOnly. Normally as signature.
/// This function is mainly used by lock script
///
pub fn parse_message() -> Result<([u8; 32], Vec<u8>), Error> {
    check_others_in_group()?;
    // Ensure that a SighashAll is present throughout the entire transaction
    let sighash_all = fetch_sighash_all()?;
    // There are 2 possible values: SighashAllOnly or SighashAll
    let witness = fetch_witness_layout()?;
    let (lock, message) = match witness.to_enum() {
        WitnessLayoutUnion::SighashAll(s) => (s.seal(), s.message()),
        WitnessLayoutUnion::SighashAllOnly(s) => (s.seal(), sighash_all.message()),
        _ => {
            return Err(Error::WrongSighashAll);
        }
    };
    let signing_message_hash = generate_signing_message_hash(message.as_slice())?;
    Ok((signing_message_hash, lock.raw_data().into()))
}
