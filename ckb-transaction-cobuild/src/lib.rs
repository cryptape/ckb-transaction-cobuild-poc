//! # CKB Transaction Cobuild Helper Library
//! This library is designed to assist developers in integrating cobuild support
//! into their scripts with ease.
//!
//! ### For Lock Script
//! Begin by implementing the `Callback` trait for your verifier:
//! ```
//! impl Callback for Verifier {
//!     fn invoke(
//!         &self,
//!         seal: &[u8],
//!         signing_message_hash: &[u8; 32],
//!     ) -> Result<(), ckb_transaction_cobuild::error::Error> {
//!         // Insert your authentication logic here
//!         Ok(())
//!     }
//! }
//! ```
//! - **`seal`**: Typically represents a signature.
//! - **`signing_message_hash`**: The hashed message that the owner signed.
//! Together with the public key/pubkey hash, these components are verified
//! using cryptographic algorithms.
//!
//! To support cobuild, proceed with calling `cobuild_entry`:
//! ```rust
//! let verifier = Verifier::new();
//! let cobuild_activated = cobuild_entry(&verifier)?;
//! ```
//! The boolean `cobuild_activated` denotes whether cobuild mode has been
//! enabled. If not, the script may default to executing its legacy code.
//!
//! ### For Type Script
//! To retrieve messages, use the `fetch_message` function. For comprehensive
//! details on utilizing messages and actions within the cobuild framework,
//! refer to the cobuild specification.
//!

#![no_std]
extern crate alloc;
pub mod blake2b;
pub mod error;
pub mod lazy_reader;
mod log;
// keep this as molecule builder is missing in lazy reader version
pub mod schemas;
// lazy reader version of molecule
pub mod schemas2;

use alloc::{collections::BTreeSet, vec::Vec};
use blake2b::{new_otx_blake2b, new_sighash_all_blake2b, new_sighash_all_only_blake2b};
use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{
        load_cell_lock_hash, load_cell_type_hash, load_script_hash, load_tx_hash, QueryIter,
    },
    syscalls,
};
use error::Error;
use lazy_reader::{new_input_cell_data, new_transaction, new_witness};
use molecule::lazy_reader::Cursor;
use schemas2::{basic, blockchain, top_level};

///
/// This is the callback trait should be implemented in lock script by
/// developers.
///
/// - **`seal`**: Typically represents a signature.
/// - **`signing_message_hash`**: The hashed message that the owner signed.
pub trait Callback {
    fn invoke(&self, seal: &[u8], signing_message_hash: &[u8; 32]) -> Result<(), Error>;
}

///
/// All script_hash in `Message.Action` should be in one of the following sets:
/// 1. all type script hashes in input cells
/// 2. all type script hashes in output cells
/// 3. all lock script hashes in input cells
///
fn check_script_hashes(hashes: BTreeSet<[u8; 32]>) -> Result<(), Error> {
    let all_hashes: BTreeSet<[u8; 32]> = QueryIter::new(load_cell_type_hash, Source::Input)
        .chain(QueryIter::new(load_cell_type_hash, Source::Output))
        .filter_map(|f| f)
        .chain(QueryIter::new(load_cell_lock_hash, Source::Input))
        .collect();
    if hashes.is_subset(&all_hashes) {
        Ok(())
    } else {
        Err(Error::ScriptHashAbsent)
    }
}

///
/// fetch the seal field of SighashAll or SighashAllOnly in current script group
///
fn fetch_seal() -> Result<Vec<u8>, Error> {
    let witness = new_witness(0, Source::GroupInput)?;
    let witness = top_level::WitnessLayout::try_from(witness)?;
    match witness {
        top_level::WitnessLayout::SighashAll(s) => {
            let seal: Vec<u8> = s.seal()?.try_into()?;
            Ok(seal)
        }
        top_level::WitnessLayout::SighashAllOnly(s) => {
            let seal: Vec<u8> = s.seal()?.try_into()?;
            Ok(seal)
        }
        _ => Err(Error::MoleculeEncoding),
    }
}

/// Retrieves the `message` field from a `SighashAll` witness.
/// - Returns `None` if a `SighashAll` witness is not present.
/// - Returns `Error::WrongWitnessLayout` if multiple `SighashAll` witnesses are
/// found. This function is intended for use within type scripts and lock
/// scripts.
pub fn fetch_message() -> Result<Option<basic::Message>, Error> {
    let tx = new_transaction();
    let (witness_layouts, _) = parse_witness_layouts(&tx)?;

    let mut iter = witness_layouts.iter().filter_map(|witness| match witness {
        Some(top_level::WitnessLayout::SighashAll(m)) => Some(m.message().unwrap().clone()),
        _ => None,
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
    let mut index = 1;
    let mut buf = [0u8; 4];
    loop {
        let r = syscalls::load_witness(&mut buf, 0, index, Source::GroupInput);
        match r {
            Ok(actual_length) => {
                if actual_length > 0 {
                    return Err(Error::WrongWitnessLayout);
                }
            }
            Err(SysError::LengthNotEnough(_)) => return Err(Error::WrongWitnessLayout),
            _ => break,
        }
        index += 1;
    }
    Ok(())
}

///
/// Generate signing message hash for SighashAll or SighashAllOnly.
///
fn generate_signing_message_hash(message: &Option<basic::Message>) -> Result<[u8; 32], Error> {
    let tx = new_transaction();

    // message
    let mut hasher = match message {
        Some(m) => {
            let mut hasher = new_sighash_all_blake2b();
            hasher.update_cursor(m.cursor.clone());
            hasher
        }
        None => new_sighash_all_only_blake2b(),
    };
    // tx hash
    hasher.update(&load_tx_hash()?);
    // inputs cell and data
    let inputs = tx.raw()?.inputs()?;
    let inputs_len = inputs.len()?;
    for i in 0..inputs_len {
        let reader = lazy_reader::InputCellReader::try_new(i, Source::Input)?;
        let cursor: Cursor = reader.into();
        hasher.update_cursor(cursor);

        let cursor = new_input_cell_data(i, Source::Input)?;
        hasher.update(&(cursor.size as u32).to_le_bytes());
        hasher.update_cursor(cursor);
    }
    // extra witnesses
    for witness in tx.witnesses()?.iter().skip(inputs_len) {
        hasher.update(&(witness.size as u32).to_le_bytes());
        hasher.update_cursor(witness);
    }
    let mut result = [0u8; 32];
    let count = hasher.count();
    hasher.finalize(&mut result);
    log!(
        "generate_signing_message_hash totally hashed {} bytes, hash = {:?}",
        count,
        result
    );
    Ok(result)
}

fn cobuild_normal_entry<F: Callback>(verifier: F) -> Result<(), Error> {
    check_others_in_group()?;
    let message = fetch_message()?;
    let signing_message_hash = generate_signing_message_hash(&message)?;
    let seal = fetch_seal()?;
    verifier.invoke(&seal, &signing_message_hash)?;
    Ok(())
}

/// Attempts to parse all witnesses into a `WitnessLayout` structure. Returns
/// `None` if parsing is not possible. For instance, parsing fails and returns
/// `None` if the structure is a `WitnessArgs`. The second return value
/// indicates whether the cobuild feature is activated (`true`) or not
/// (`false`).
fn parse_witness_layouts(
    tx: &blockchain::Transaction,
) -> Result<(Vec<Option<top_level::WitnessLayout>>, bool), Error> {
    let witness_layouts: Vec<Option<top_level::WitnessLayout>> = tx
        .witnesses()?
        .into_iter()
        .map(|w| top_level::WitnessLayout::try_from(w).ok())
        .collect();
    let mut activated = false;
    for w in &witness_layouts {
        if let Some(w2) = w {
            w2.verify(false)?;
            activated = true;
        }
    }
    Ok((witness_layouts, activated))
}

/// Serves as the primary entry point for a lock script supporting cobuild.
/// Operates in conjunction with the `Callback` trait. For integration
/// instructions into cobuild, refer to the crate documentation.
pub fn cobuild_entry<F: Callback>(verifier: F) -> Result<bool, Error> {
    let tx = new_transaction();
    let raw_tx = tx.raw()?;
    let (witness_layouts, cobuild_activated) = parse_witness_layouts(&tx)?;
    // Legacy Flow Handling
    if !cobuild_activated {
        return Ok(false);
    }

    let current_script_hash = load_script_hash()?;
    // step 2
    // step 4
    let (otx_start, i) = fetch_otx_start(&witness_layouts)?;
    if otx_start.is_none() {
        // step 3
        log!("No otx detected");
        cobuild_normal_entry(verifier)?;
        return Ok(true);
    }
    let otx_start = otx_start.unwrap();

    let start_input_cell: u32 = otx_start.start_input_cell()?;
    let start_output_cell: u32 = otx_start.start_output_cell()?;
    let start_cell_deps: u32 = otx_start.start_cell_deps()?;
    let start_header_deps: u32 = otx_start.start_header_deps()?;
    // abbrev. from spec:
    // ie = input end
    // is = input start
    // oe = output end
    // os = output start
    // ce = cell deps end
    // cs = cell deps start
    // he = header dep end
    // hs = header dep start
    // step 5
    let mut ie = start_input_cell as usize;
    let is = ie;
    let mut oe = start_output_cell as usize;
    let mut ce = start_cell_deps as usize;
    let mut he = start_header_deps as usize;
    let mut execution_count: usize = 0;
    let mut otx_count = 0;
    log!("ie = {}, oe = {}, ce = {}, he = {}", ie, oe, ce, he);
    log!("Otx starts at index {}(inclusive)", i + 1);
    // this index is always pointing to the current processing OTX witness.
    let mut index = i;
    for witness_index in i + 1..witness_layouts.len() {
        index = witness_index;
        let witness = witness_layouts.get(witness_index).unwrap();
        if witness.is_none() {
            // step 6, not WitnessLayoutOtx
            break;
        }
        match witness {
            Some(top_level::WitnessLayout::Otx(ref otx)) => {
                otx_count += 1;
                let input_cells: u32 = otx.input_cells()?;
                let output_cells: u32 = otx.output_cells()?;
                let cell_deps: u32 = otx.cell_deps()?;
                let header_deps: u32 = otx.header_deps()?;
                // step 6.b
                if input_cells == 0 && output_cells == 0 && cell_deps == 0 && header_deps == 0 {
                    return Err(Error::WrongCount);
                }
                let action_hashes: BTreeSet<[u8; 32]> = otx
                    .message()?
                    .actions()?
                    .iter()
                    .map(|pair| pair.script_hash().unwrap())
                    .collect();
                // step 6.c
                check_script_hashes(action_hashes)?;
                // step 6.d
                let lock_hash_existing = QueryIter::new(load_cell_lock_hash, Source::Input)
                    .skip(ie)
                    .take(input_cells as usize)
                    .any(|hash| hash == current_script_hash);
                if !lock_hash_existing {
                    ie += input_cells as usize;
                    oe += output_cells as usize;
                    ce += cell_deps as usize;
                    he += header_deps as usize;
                    continue;
                }
                // step 6.e
                let smh = generate_otx_smh(&otx, &raw_tx, ie, oe, ce, he)?;
                // step 6.f
                let mut seal_found = false;
                for seal_pair in otx.seals()?.iter() {
                    if seal_pair.script_hash()? == current_script_hash.as_slice() {
                        let seal: Vec<u8> = seal_pair.seal()?.try_into()?;
                        log!("invoke OTX verifier");
                        verifier.invoke(&seal, &smh)?;
                        seal_found = true;
                        execution_count += 1;
                        break;
                        // duplicated seals are ignored
                    }
                }
                if !seal_found {
                    log!("seal can't be found");
                    return Err(Error::NoSealFound);
                }
                // step 6.h
                ie += input_cells as usize;
                oe += output_cells as usize;
                ce += cell_deps as usize;
                he += header_deps as usize;
            }
            _ => {
                break;
            }
        }
    } // end of step 6 loop

    // step 7
    // after the loop, the j points to the first non OTX witness or out of bounds
    let j = if index == (witness_layouts.len() - 1) {
        witness_layouts.len()
    } else {
        index
    };
    log!("the first non OTX witness is at index {}", j);
    for loop_index in 0..witness_layouts.len() {
        // [0, i) [j, +infinity)
        if loop_index < i || loop_index >= j {
            if let Some(r) = &witness_layouts.get(loop_index) {
                match r {
                    Some(top_level::WitnessLayout::Otx(_)) => {
                        log!(
                            "WrongWitnessLayout at index = {} (i = {}, j = {}, otx_count = {})",
                            loop_index,
                            i,
                            j,
                            otx_count
                        );
                        return Err(Error::WrongWitnessLayout);
                    }
                    _ => {}
                }
            }
        }
    }
    // step 8
    let mut found = false;
    for index in 0..raw_tx.inputs()?.len()? {
        // scan all input cell in [0, is) and [ie, +infinity)
        // if is == ie, it is always true
        if index < is || index >= ie {
            let hash = load_cell_lock_hash(index, Source::Input)?;
            if hash == current_script_hash {
                found = true;
                break;
            }
        }
    }
    if found {
        execution_count += 1;
        log!("extra callback is invoked");
        cobuild_normal_entry(verifier)?;
    }
    log!("execution_count = {}", execution_count);
    Ok(true)
}

/// generate OTX signing message hash
fn generate_otx_smh(
    otx: &basic::Otx,
    raw_tx: &blockchain::RawTransaction,
    ie: usize,
    oe: usize,
    ce: usize,
    he: usize,
) -> Result<[u8; 32], Error> {
    log!("ie = {}, oe = {}, ce = {}, he = {}", ie, oe, ce, he);
    let input_cells: u32 = otx.input_cells()?;
    let output_cells: u32 = otx.output_cells()?;
    let cell_deps: u32 = otx.cell_deps()?;
    let header_deps: u32 = otx.header_deps()?;

    let mut hasher = new_otx_blake2b();
    hasher.update_cursor(otx.message()?.cursor.clone());
    hasher.update(&input_cells.to_le_bytes());

    let inputs = raw_tx.inputs()?;
    for index in ie..ie + input_cells as usize {
        // input
        hasher.update_cursor(inputs.get(index)?.cursor);

        let reader = lazy_reader::InputCellReader::try_new(index, Source::Input)?;
        let cursor: Cursor = reader.into();
        let data_cursor = new_input_cell_data(index, Source::Input)?;
        // input cell
        hasher.update_cursor(cursor);
        // input cell data size
        hasher.update(&(data_cursor.size as u32).to_le_bytes());
        // input cell data
        hasher.update_cursor(data_cursor);
    }

    hasher.update(&output_cells.to_le_bytes());

    for index in oe..oe + output_cells as usize {
        let outputs = raw_tx.outputs()?;
        let outputs_data = raw_tx.outputs_data()?;
        // output cell
        hasher.update_cursor(outputs.get(index)?.cursor);
        let data = outputs_data.get(index)?;
        // output cell data size
        hasher.update(&(data.size as u32).to_le_bytes());
        // output cell data
        hasher.update_cursor(data);
    }

    hasher.update(&cell_deps.to_le_bytes());

    for index in ce..ce + cell_deps as usize {
        let cell_deps = raw_tx.cell_deps()?;
        hasher.update_cursor(cell_deps.get(index)?.cursor)
    }

    hasher.update(&header_deps.to_le_bytes());

    for index in he..he + header_deps as usize {
        let header_deps = raw_tx.header_deps()?;
        hasher.update(&header_deps.get(index)?);
    }

    let mut result = [0u8; 32];
    let count = hasher.count();
    hasher.finalize(&mut result);
    log!(
        "generate_otx_smh totally hashed {} bytes and hash is {:?}",
        count,
        result
    );
    Ok(result)
}

///
/// parse all witnesses and find out the `OtxStart`
///
fn fetch_otx_start(
    witnesses: &Vec<Option<top_level::WitnessLayout>>,
) -> Result<(Option<basic::OtxStart>, usize), Error> {
    let mut otx_start = None;
    let mut start_index = 0;
    let mut end_index = 0;

    for (i, witness) in witnesses.iter().enumerate() {
        if let Some(witness_layout) = witness {
            match witness_layout {
                top_level::WitnessLayout::OtxStart(start) => {
                    if otx_start.is_none() {
                        otx_start = Some(start.clone());
                        start_index = i;
                        end_index = i;
                    } else {
                        log!("Duplicated OtxStart found");
                        return Err(Error::WrongWitnessLayout);
                    }
                }
                top_level::WitnessLayout::Otx(_) => {
                    if otx_start.is_none() {
                        log!("A Otx without OtxStart found");
                        return Err(Error::WrongWitnessLayout);
                    } else {
                        if end_index + 1 != i {
                            log!("Otx are not continuous");
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

    if otx_start.is_some() {
        if end_index > 0 {
            return Ok((otx_start, start_index));
        } else {
            log!("end_index == 0, there is no OTX");
            return Err(Error::WrongOtxStart);
        }
    } else {
        Ok((None, 0))
    }
}
