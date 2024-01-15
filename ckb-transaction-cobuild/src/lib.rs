#![no_std]
extern crate alloc;
pub mod blake2b;
pub mod schemas;

use alloc::vec::Vec;
use blake2b::{new_otx_blake2b, new_sighash_all_blake2b, new_sighash_all_only_blake2b};
use ckb_gen_types::prelude::Unpack;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed::{CellInput, Transaction},
    error::SysError,
    high_level::{
        self, load_cell, load_cell_data, load_cell_lock_hash, load_tx_hash, load_witness, QueryIter,
    },
    syscalls::load_transaction,
};
use core::convert::Into;
use molecule::{
    error::VerificationError,
    prelude::{Entity, Reader},
    NUMBER_SIZE,
};
use schemas::{
    basic::{Message, OtxStart, SealPairVec},
    top_level::{WitnessLayoutReader, WitnessLayoutUnionReader},
};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Error {
    Sys(SysError),
    MoleculeEncoding,
    WrongSighashAll,
    WrongWitnessLayout,
    WrongOtxStart,
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
/// fetch the seal field of SighashAll or SighashAllOnly in current script group
///
fn fetch_seal() -> Result<Vec<u8>, Error> {
    match load_witness(0, Source::GroupInput) {
        Ok(witness) => {
            if let Ok(r) = WitnessLayoutReader::from_slice(&witness) {
                match r.to_enum() {
                    WitnessLayoutUnionReader::SighashAll(s) => Ok(s.seal().raw_data().to_vec()),
                    WitnessLayoutUnionReader::SighashAllOnly(s) => Ok(s.seal().raw_data().to_vec()),
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
/// fetch the message field of SighashAll
/// returns None if there is no SighashAll witness
/// returns Error::WrongWitnessLayout if there are more than one SighashAll witness
pub fn fetch_message() -> Result<Option<Message>, Error> {
    let mut iter = QueryIter::new(load_witness, Source::Input).filter_map(|witness| {
        WitnessLayoutReader::from_slice(&witness)
            .ok()
            .and_then(|r| match r.to_enum() {
                WitnessLayoutUnionReader::SighashAll(s) => Some(s.message().to_entity()),
                _ => None,
            })
    });

    match (iter.next(), iter.next()) {
        (Some(message), None) => Ok(Some(message)),
        (None, None) => Ok(None),
        _ => Err(Error::WrongWitnessLayout),
    }
}

///
/// for lock script with message, the other witness in script group except
/// first one should be empty
///
fn check_others_in_group() -> Result<(), Error> {
    if QueryIter::new(load_witness, Source::GroupInput)
        .skip(1)
        .all(|witness| witness.is_empty())
    {
        Ok(())
    } else {
        Err(Error::WrongWitnessLayout)
    }
}

fn generate_signing_message_hash(message: &Option<Message>) -> Result<[u8; 32], Error> {
    // message
    let mut hasher = match message {
        Some(m) => {
            let mut hasher = new_sighash_all_blake2b();
            hasher.update(m.as_slice());
            hasher
        }
        None => new_sighash_all_only_blake2b(),
    };
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
    Ok(result)
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
    let message = fetch_message()?;
    let signing_message_hash = generate_signing_message_hash(&message)?;
    let seal = fetch_seal()?;
    Ok((signing_message_hash, seal))
}

/// OtxMessageIter is an iterator over the otx message in current transaction
/// The item of this iterator is a tuple of signing_message_hash and SealPairVec
pub struct OtxMessageIter {
    tx: Transaction,
    current_script_hash: [u8; 32],
    witness_counter: usize,
    input_cell_counter: usize,
    output_cell_counter: usize,
    cell_deps_counter: usize,
    header_deps_counter: usize,
}

impl Iterator for OtxMessageIter {
    type Item = ([u8; 32], SealPairVec);

    fn next(&mut self) -> Option<Self::Item> {
        let witness_iter = self.tx.witnesses().into_iter().skip(self.witness_counter);
        let raw_tx = self.tx.raw();
        for witness in witness_iter {
            if let Ok(r) = WitnessLayoutReader::from_slice(&witness.raw_data()) {
                match r.to_enum() {
                    WitnessLayoutUnionReader::Otx(otx) => {
                        self.witness_counter += 1;
                        let input_cells: u32 = otx.input_cells().unpack();
                        let output_cells: u32 = otx.output_cells().unpack();
                        let cell_deps: u32 = otx.cell_deps().unpack();
                        let header_deps: u32 = otx.header_deps().unpack();
                        let mut input_lock_hash_iter =
                            QueryIter::new(load_cell_lock_hash, Source::Input)
                                .skip(self.input_cell_counter)
                                .take(input_cells as usize);
                        if input_lock_hash_iter
                            .any(|lock_hash| lock_hash == self.current_script_hash)
                        {
                            let mut hasher = new_otx_blake2b();
                            // message
                            hasher.update(otx.message().as_slice());

                            // otx inputs
                            hasher.update(&input_cells.to_le_bytes());
                            let input_iter = raw_tx
                                .inputs()
                                .into_iter()
                                .skip(self.input_cell_counter)
                                .zip(
                                    QueryIter::new(load_cell, Source::Input)
                                        .skip(self.input_cell_counter),
                                )
                                .zip(
                                    QueryIter::new(load_cell_data, Source::Input)
                                        .skip(self.input_cell_counter),
                                );
                            for ((input, input_cell), input_cell_data) in
                                input_iter.take(input_cells as usize)
                            {
                                hasher.update(input.as_slice());
                                hasher.update(input_cell.as_slice());
                                hasher.update(&(input_cell_data.len() as u32).to_le_bytes());
                                hasher.update(&input_cell_data);
                            }
                            self.input_cell_counter += input_cells as usize;

                            // otx outputs
                            hasher.update(&output_cells.to_le_bytes());
                            let output_iter = raw_tx
                                .outputs()
                                .into_iter()
                                .skip(self.output_cell_counter)
                                .zip(
                                    raw_tx
                                        .outputs_data()
                                        .into_iter()
                                        .skip(self.output_cell_counter),
                                );
                            for (output_cell, output_cell_data) in
                                output_iter.take(output_cells as usize)
                            {
                                hasher.update(output_cell.as_slice());
                                // according to the spec, we need to hash the output data length first in little endian, then the data itself.
                                // we are using molecule serialized slice directly here, it's same as the spec.
                                hasher.update(output_cell_data.as_slice());
                            }
                            self.output_cell_counter += output_cells as usize;

                            // otx cell deps
                            hasher.update(&cell_deps.to_le_bytes());
                            let cell_dep_iter =
                                raw_tx.cell_deps().into_iter().skip(self.cell_deps_counter);
                            for cell_dep in cell_dep_iter.take(cell_deps as usize) {
                                hasher.update(cell_dep.as_slice());
                            }
                            self.cell_deps_counter += cell_deps as usize;

                            // otx header deps
                            hasher.update(&header_deps.to_le_bytes());
                            let header_dep_iter = raw_tx
                                .header_deps()
                                .into_iter()
                                .skip(self.header_deps_counter);
                            for header_dep in header_dep_iter.take(header_deps as usize) {
                                hasher.update(header_dep.as_slice());
                            }
                            self.header_deps_counter += header_deps as usize;

                            let mut result = [0u8; 32];
                            hasher.finalize(&mut result);
                            return Some((result, otx.seals().to_entity()));
                        } else {
                            self.input_cell_counter += input_cells as usize;
                            self.output_cell_counter += output_cells as usize;
                            self.cell_deps_counter += cell_deps as usize;
                            self.header_deps_counter += header_deps as usize;
                        }
                    }
                    _ => return None,
                }
            } else {
                return None;
            }
        }

        None
    }
}

///
/// verify all otx messages with the given script hash and verify function
/// This function is mainly used by lock script
///
pub fn verify_otx_message<F: Fn(&[u8], &[u8; 32]) -> bool>(
    current_script_hash: [u8; 32],
    verify: F,
) -> Result<bool, Error> {
    let mut otx_message_iter = parse_otx_message(current_script_hash)?;
    let verified = otx_message_iter.all(|(message_digest, seals)| {
        seals
            .into_iter()
            .filter(|seal_pair| {
                seal_pair.script_hash().as_slice() == current_script_hash.as_slice()
            })
            .any(|seal_pair| verify(&seal_pair.seal().raw_data(), &message_digest))
    });
    Ok(verified)
}

///
/// parse transaction and return `OtxMessageIter`
/// This function is mainly used by lock script
///
pub fn parse_otx_message(current_script_hash: [u8; 32]) -> Result<OtxMessageIter, Error> {
    let (otx_start, start_index) = fetch_otx_start()?;
    let start_input_cell: u32 = otx_start.start_input_cell().unpack();
    let start_output_cell: u32 = otx_start.start_output_cell().unpack();
    let start_cell_deps: u32 = otx_start.start_cell_deps().unpack();
    let start_header_deps: u32 = otx_start.start_header_deps().unpack();

    let tx = high_level::load_transaction()?;

    Ok(OtxMessageIter {
        tx,
        current_script_hash,
        witness_counter: start_index + 1,
        input_cell_counter: start_input_cell as usize,
        output_cell_counter: start_output_cell as usize,
        cell_deps_counter: start_cell_deps as usize,
        header_deps_counter: start_header_deps as usize,
    })
}

fn fetch_otx_start() -> Result<(OtxStart, usize), Error> {
    let mut otx_start = None;
    let mut start_index = 0;
    let mut end_index = 0;

    for (i, witness) in QueryIter::new(load_witness, Source::Input).enumerate() {
        if let Ok(r) = WitnessLayoutReader::from_slice(&witness) {
            match r.to_enum() {
                WitnessLayoutUnionReader::OtxStart(o) => {
                    if otx_start.is_none() {
                        otx_start = Some(o.to_entity());
                        start_index = i;
                        end_index = i;
                    } else {
                        return Err(Error::WrongWitnessLayout);
                    }
                }
                WitnessLayoutUnionReader::Otx(_) => {
                    if otx_start.is_none() {
                        return Err(Error::WrongWitnessLayout);
                    } else {
                        if end_index + 1 != i {
                            return Err(Error::WrongWitnessLayout);
                        } else {
                            end_index = i;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(otx_start) = otx_start {
        if end_index > 0 {
            return Ok((otx_start, start_index));
        }
    }
    Err(Error::WrongOtxStart)
}
