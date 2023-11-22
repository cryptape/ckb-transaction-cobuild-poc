#![no_std]
extern crate alloc;
pub mod blake2b;
pub mod schemas;

use alloc::vec::Vec;
use blake2b::new_blake2b;
use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_input_since, load_tx_hash, load_witness},
};
use molecule::{
    error::VerificationError,
    prelude::{Entity, Reader},
};
use schemas::{
    basic::SighashWithAction,
    top_level::{
        ExtendedWitness, ExtendedWitnessReader, ExtendedWitnessUnion, ExtendedWitnessUnionReader,
    },
};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Error {
    Sys(SysError),
    DuplicateAction,
    MoleculeEncoding,
    WrongSighashWithAction,
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
/// A single transaction must have only one SighashWithAction
///
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
        return Err(Error::WrongSighashWithAction);
    }
}

///
/// Fetch the corresponding ExtendedWitness( Sighash or SighashWithAction)
/// Used by lock script
///
pub fn fetch_sighash() -> Result<ExtendedWitness, Error> {
    match load_witness(0, Source::GroupInput) {
        Ok(witness) => {
            if let Ok(r) = ExtendedWitnessReader::from_slice(&witness) {
                match r.to_enum() {
                    ExtendedWitnessUnionReader::SighashWithAction(_)
                    | ExtendedWitnessUnionReader::Sighash(_) => Ok(r.to_entity()),
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
/// Search the only SighashWithAction from all witnesses.
///
pub fn search_sighash_with_action() -> Result<SighashWithAction, Error> {
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
        return Err(Error::WrongSighashWithAction);
    }
}

//
// Rule for hashing:
// 1. Variable length data should hash the length.
// 2. Fixed length data don't need to hash the length.
//
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

pub fn generate_final_hash(skeleton_hash: &[u8; 32], typed_message: &[u8]) -> [u8; 32] {
    let mut hasher = new_blake2b();
    hasher.update(&skeleton_hash[..]);
    hasher.update(&(typed_message.len() as u64).to_le_bytes());
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

///
/// parse transaction with typed message and return 2 values:
/// 1. digest message, 32 bytes message for signature verification
/// 2. lock, lock field in SighashWithAction or Sighash. Normally as signature.
/// This function is mainly used by lock script
///
pub fn parse_typed_message() -> Result<([u8; 32], Vec<u8>), Error> {
    // Ensure that a SighashWitAction is present throughout the entire transaction
    check_sighash_with_action()?;
    // There are 2 possible values: Sighash or SighashWithAction
    let witness = fetch_sighash()?;
    let (lock, typed_message) = match witness.to_enum() {
        ExtendedWitnessUnion::SighashWithAction(s) => (s.lock(), s.message()),
        ExtendedWitnessUnion::Sighash(s) => {
            let sighash_with_action = search_sighash_with_action()?;
            (s.lock(), sighash_with_action.message())
        }
        _ => {
            return Err(Error::WrongSighashWithAction);
        }
    };
    let skeleton_hash = generate_skeleton_hash()?;
    let digest_message = generate_final_hash(&skeleton_hash, typed_message.as_slice());
    let lock = lock.as_slice();
    Ok((digest_message, lock.to_vec()))
}
