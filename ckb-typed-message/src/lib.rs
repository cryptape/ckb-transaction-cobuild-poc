#![no_std]
pub mod blake2b;
pub mod schemas;

use blake2b::new_blake2b;
use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_input_since, load_tx_hash, load_witness},
};
use molecule::{error::VerificationError, prelude::Reader};
use schemas::{
    basic::SighashWithAction,
    top_level::{ExtendedWitness, ExtendedWitnessReader, ExtendedWitnessUnionReader},
};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Error {
    Sys(SysError),
    DuplicateAction,
    MoleculeEncoding,
    NotTypedTransaction,
    NoSighashWithAction,
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

/// A single transaction must only have one SighashWithAction
pub fn check_sighash_with_action() -> Result<(), Error> {
    let mut i = 0;
    let mut found = false;
    loop {
        match load_witness(i, Source::Input) {
            Ok(witness) => {
                if let Ok(r) = ExtendedWitnessReader::from_slice(&witness) {
                    if let ExtendedWitnessUnionReader::SighashWithAction(_) = r.to_enum() {
                        if found {
                            return Err(Error::DuplicateAction);
                        } else {
                            found = true;
                        }
                    }
                }
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(e) => return Err(e.into()),
        }
        i += 1;
    }
    if found {
        return Ok(());
    } else {
        return Err(Error::NoSighashWithAction);
    }
}

///
/// For a lock script with typed message support, the returned value must be
/// SighashWithAction or Sighash
///
pub fn get_lock_sighash() -> Result<ExtendedWitness, Error> {
    check_sighash_with_action()?;
    match load_witness(0, Source::GroupInput) {
        Ok(witness) => {
            if let Ok(r) = ExtendedWitnessReader::from_slice(&witness) {
                if let ExtendedWitnessUnionReader::SighashWithAction(_) = r.to_enum() {
                    return Ok(r.to_entity());
                } else if let ExtendedWitnessUnionReader::Sighash(_) = r.to_enum() {
                    return Ok(r.to_entity());
                } else {
                    return Err(Error::MoleculeEncoding);
                }
            } else {
                return Err(Error::MoleculeEncoding);
            }
        }
        Err(e) => return Err(e.into()),
    }
}

///
/// For a type script with typed message support, the returned value must
/// be SighashWithAction with no duplication.
///
pub fn get_type_sighash_with_action() -> Result<SighashWithAction, Error> {
    check_sighash_with_action()?;
    let mut i = 0;
    let mut result = None;
    // Look for the first SighashWithAction witness
    while result.is_none() {
        match load_witness(i, Source::Input) {
            Ok(witness) => {
                if let Ok(r) = ExtendedWitnessReader::from_slice(&witness) {
                    if let ExtendedWitnessUnionReader::SighashWithAction(s) = r.to_enum() {
                        result = Some(s.to_entity());
                    }
                }
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(e) => return Err(e.into()),
        };
        i += 1;
    }
    if result.is_some() {
        return Ok(result.unwrap());
    } else {
        return Err(Error::NoSighashWithAction);
    }
}

pub fn generate_skeleton_hash() -> Result<[u8; 32], Error> {
    let mut hasher = new_blake2b();
    hasher.update(&load_tx_hash()?);

    let mut i = calculate_inputs_len()?;
    loop {
        match load_witness(i, Source::Input) {
            Ok(w) => {
                hasher.update(&(w.len() as u64).to_le_bytes());
                hasher.update(&w);
            }
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(e) => return Err(e.into()),
        }
        i += 1;
    }

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    Ok(output)
}

pub fn calculate_final_hash(skeleton_hash: &[u8; 32], typed_message: &[u8]) -> [u8; 32] {
    let mut hasher = new_blake2b();
    hasher.update(&skeleton_hash[..]);
    hasher.update(typed_message);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    return output;
}

// Translated from https://github.com/nervosnetwork/ckb-system-scripts/blob/a7b7c75662ed950c9bd024e15f83ce702a54996e/c/common.h#L32-L66
fn calculate_inputs_len() -> Result<usize, SysError> {
    let mut lo = 0;
    let mut hi = 4;

    // The code below can't handle the scenario when input length is zero.
    let first_available = load_input_since(0, Source::Input);
    if first_available.is_err() {
        return Ok(0);
    }

    loop {
        match load_input_since(hi, Source::Input) {
            Ok(_) => {
                lo = hi;
                hi *= 2;
            }
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(e) => return Err(e),
        }
    }

    while (lo + 1) != hi {
        let i = (lo + hi) / 2;
        match load_input_since(i, Source::Input) {
            Ok(_) => {
                lo = i;
            }
            Err(SysError::IndexOutOfBound) => {
                hi = i;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(hi)
}
