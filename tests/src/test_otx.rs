use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack, Unpack};
use molecule::prelude::Reader;
use rand::seq::SliceRandom;
use rand::RngCore;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use ckb_testtool::{
    ckb_chain_spec, ckb_crypto, ckb_error, ckb_hash, ckb_jsonrpc_types, ckb_script, ckb_traits,
    ckb_types,
};
use ckb_transaction_cobuild::schemas;

lazy_static::lazy_static! {
    static ref BINARY_ALWAYS_SUCCESS: ckb_types::bytes::Bytes = super::Loader::default().load_binary("always-success");
    static ref BINARY_TRANSACTION_COBUILD_LOCK_DEMO: ckb_types::bytes::Bytes = super::Loader::default().load_binary("transaction-cobuild-lock-demo");
    static ref BINARY_AUTH: ckb_types::bytes::Bytes = super::Loader::default().load_binary("../auth");
    static ref BINARY_SECP256K1_DATA: ckb_types::bytes::Bytes = super::Loader::default().load_binary("../secp256k1_data_20210801");
}

#[derive(Clone, Default)]
pub struct Resource {
    pub cell: HashMap<ckb_types::packed::OutPoint, ckb_types::core::cell::CellMeta>,
}

impl ckb_traits::CellDataProvider for Resource {
    fn get_cell_data(
        &self,
        out_point: &ckb_types::packed::OutPoint,
    ) -> Option<ckb_types::bytes::Bytes> {
        self.cell
            .get(out_point)
            .and_then(|cell_meta| cell_meta.mem_cell_data.clone())
    }

    fn get_cell_data_hash(
        &self,
        out_point: &ckb_types::packed::OutPoint,
    ) -> Option<ckb_types::packed::Byte32> {
        self.cell
            .get(out_point)
            .and_then(|cell_meta| cell_meta.mem_cell_data_hash.clone())
    }
}

impl ckb_traits::HeaderProvider for Resource {
    fn get_header(&self, _: &ckb_types::packed::Byte32) -> Option<ckb_types::core::HeaderView> {
        unimplemented!()
    }
}

impl ckb_traits::ExtensionProvider for Resource {
    fn get_block_extension(
        &self,
        _: &ckb_types::packed::Byte32,
    ) -> Option<ckb_types::packed::Bytes> {
        unimplemented!()
    }
}

impl ckb_types::core::cell::CellProvider for Resource {
    fn cell(
        &self,
        out_point: &ckb_types::packed::OutPoint,
        eager_load: bool,
    ) -> ckb_types::core::cell::CellStatus {
        let _ = eager_load;
        if let Some(data) = self.cell.get(out_point).cloned() {
            ckb_types::core::cell::CellStatus::Live(data)
        } else {
            ckb_types::core::cell::CellStatus::Unknown
        }
    }
}

impl ckb_types::core::cell::HeaderChecker for Resource {
    fn check_valid(
        &self,
        _: &ckb_types::packed::Byte32,
    ) -> Result<(), ckb_types::core::error::OutPointError> {
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct Verifier {}

impl Verifier {
    pub fn verify_prior(
        &self,
        tx_resolved: &ckb_types::core::cell::ResolvedTransaction,
        _: &Resource,
    ) {
        let a = tx_resolved.transaction.outputs().item_count();
        let b = tx_resolved.transaction.outputs_data().item_count();
        assert_eq!(a, b);
    }

    pub fn verify(
        &self,
        tx_resolved: &ckb_types::core::cell::ResolvedTransaction,
        dl: &Resource,
    ) -> Result<ckb_types::core::Cycle, ckb_error::Error> {
        self.verify_prior(&tx_resolved, &dl);
        let hardfork = ckb_types::core::hardfork::HardForks {
            ckb2021: ckb_types::core::hardfork::CKB2021::new_mirana()
                .as_builder()
                .rfc_0032(10)
                .build()
                .unwrap(),
            ckb2023: ckb_types::core::hardfork::CKB2023::new_mirana()
                .as_builder()
                .rfc_0049(20)
                .build()
                .unwrap(),
        };
        let consensus = ckb_chain_spec::consensus::ConsensusBuilder::default()
            .hardfork_switch(hardfork)
            .build();
        let mut verifier = ckb_script::TransactionScriptsVerifier::new(
            Arc::new(tx_resolved.clone()),
            dl.clone(),
            Arc::new(consensus),
            Arc::new(ckb_script::TxVerifyEnv::new_commit(
                &ckb_types::core::HeaderView::new_advanced_builder()
                    .epoch(ckb_types::core::EpochNumberWithFraction::new(10, 0, 1).pack())
                    .build(),
            )),
        );
        verifier.set_debug_printer(|script: &ckb_types::packed::Byte32, msg: &str| {
            let str = format!("Script({})", hex::encode(&script.as_slice()[..4]));
            println!("{}: {}", str, msg);
        });
        let result = verifier.verify(u64::MAX);
        if result.is_ok() {
            let cycles = (*result.as_ref().unwrap() as f64) / 1024.0 / 1024.0;
            println!("cycles = {:.1} M ", cycles);
        }
        result
    }
}

#[derive(Clone, Default)]
pub struct Pickaxer {
    outpoint_hash: ckb_types::packed::Byte32,
    outpoint_i: u32,
}

impl Pickaxer {
    pub fn insert_cell_data(
        &mut self,
        dl: &mut Resource,
        data: &ckb_types::bytes::Bytes,
    ) -> ckb_types::core::cell::CellMeta {
        let cell_out_point =
            ckb_types::packed::OutPoint::new(self.outpoint_hash.clone(), self.outpoint_i);
        let cell_output = ckb_types::packed::CellOutput::new_builder()
            .capacity(ckb_types::core::Capacity::bytes(0).unwrap().pack())
            .build();
        let cell_data = ckb_types::bytes::Bytes::copy_from_slice(data);
        let cell_meta =
            ckb_types::core::cell::CellMetaBuilder::from_cell_output(cell_output, cell_data)
                .out_point(cell_out_point.clone())
                .build();
        dl.cell.insert(cell_out_point.clone(), cell_meta.clone());
        self.outpoint_i += 1;
        cell_meta
    }

    pub fn insert_cell_fund(
        &mut self,
        dl: &mut Resource,
        lock: ckb_types::packed::Script,
        kype: Option<ckb_types::packed::Script>,
        data: &[u8],
    ) -> ckb_types::core::cell::CellMeta {
        let cell_out_point =
            ckb_types::packed::OutPoint::new(self.outpoint_hash.clone(), self.outpoint_i);
        let cell_output = ckb_types::packed::CellOutput::new_builder()
            .capacity(ckb_types::core::Capacity::bytes(0).unwrap().pack())
            .lock(lock)
            .type_(
                ckb_types::packed::ScriptOpt::new_builder()
                    .set(kype)
                    .build(),
            )
            .build();
        let cell_data = ckb_types::bytes::Bytes::copy_from_slice(data);
        let cell_meta =
            ckb_types::core::cell::CellMetaBuilder::from_cell_output(cell_output, cell_data)
                .out_point(cell_out_point.clone())
                .build();
        dl.cell.insert(cell_out_point.clone(), cell_meta.clone());
        self.outpoint_i += 1;
        cell_meta
    }

    pub fn create_cell_dep(
        &self,
        cell_meta: &ckb_types::core::cell::CellMeta,
    ) -> ckb_types::packed::CellDep {
        ckb_types::packed::CellDep::new_builder()
            .out_point(cell_meta.out_point.clone())
            .dep_type(ckb_types::core::DepType::Code.into())
            .build()
    }

    pub fn create_cell_input(
        &self,
        cell_meta: &ckb_types::core::cell::CellMeta,
    ) -> ckb_types::packed::CellInput {
        ckb_types::packed::CellInput::new(cell_meta.out_point.clone(), 0)
    }

    pub fn create_cell_output(
        &self,
        lock: ckb_types::packed::Script,
        kype: Option<ckb_types::packed::Script>,
    ) -> ckb_types::packed::CellOutput {
        ckb_types::packed::CellOutput::new_builder()
            .capacity(ckb_types::core::Capacity::bytes(0).unwrap().pack())
            .lock(lock)
            .type_(
                ckb_types::packed::ScriptOpt::new_builder()
                    .set(kype)
                    .build(),
            )
            .build()
    }

    pub fn create_script(
        &self,
        cell_meta: &ckb_types::core::cell::CellMeta,
        args: &[u8],
    ) -> ckb_types::packed::Script {
        ckb_types::packed::Script::new_builder()
            .args(args.pack())
            .code_hash(cell_meta.mem_cell_data_hash.clone().unwrap())
            .hash_type(ckb_types::core::ScriptHashType::Data1.into())
            .build()
    }
}

pub fn println_hex(name: &str, data: &[u8]) {
    println!(
        "Tester(........): {}(len={}): {}",
        name,
        data.len(),
        hex::encode(data)
    );
}

pub fn println_log(data: &str) {
    println!("Tester(........): {}", data);
}

pub fn _println_rtx(tx_resolved: &ckb_types::core::cell::ResolvedTransaction) {
    let tx_json = ckb_jsonrpc_types::TransactionView::from(tx_resolved.transaction.clone());
    println!(
        "Tester(........): {}",
        serde_json::to_string_pretty(&tx_json).unwrap()
    );
}

pub fn get_ckb_key(data: &str) -> (ckb_crypto::secp::Privkey, [u8; 20]) {
    let prikey = ckb_crypto::secp::Privkey::from_str(data).unwrap();
    let pubkey = prikey.pubkey().unwrap();
    let pubkey_hash = ckb_hash::blake2b_256(&pubkey.serialize());

    (prikey, pubkey_hash[0..20].to_vec().try_into().unwrap())
}

pub fn sign_pubkey_hash(prikey: ckb_crypto::secp::Privkey, message: &[u8]) -> Vec<u8> {
    assert_eq!(message.len(), 32);
    let sign = ckb_types::H256::from_slice(message).unwrap();
    let sign = prikey.sign_recoverable(&sign).unwrap().serialize();
    sign
}

pub fn cobuild_create_signing_message_hash_sighash_all_only(
    tx: ckb_types::core::TransactionView,
    dl: &Resource,
) -> Vec<u8> {
    let mut hasher = ckb_hash::Blake2bBuilder::new(32)
        .personal(b"ckb-tcob-sgohash")
        .build();
    hasher.update(tx.hash().as_slice());
    let inputs_len = tx.inputs().len();
    for i in 0..inputs_len {
        let input_cell = tx.inputs().get(i).unwrap();
        let input_cell_out_point = input_cell.previous_output();
        let input_cell_meta = dl.cell.get(&input_cell_out_point).unwrap();
        hasher.update(input_cell_meta.cell_output.as_slice());
        hasher.update(&(input_cell_meta.data_bytes as u32).to_le_bytes());
        hasher.update(&input_cell_meta.mem_cell_data.clone().unwrap());
    }
    for witness in tx.witnesses().into_iter().skip(inputs_len) {
        hasher.update(&(witness.len() as u32).to_le_bytes());
        hasher.update(&witness.raw_data());
    }
    let mut result = vec![0u8; 32];
    hasher.finalize(&mut result);
    result
}

pub fn cobuild_create_signing_message_hash_otx(
    tx: ckb_types::core::TransactionView,
    dl: &Resource,
    message: &schemas::basic::Message,
) -> Vec<u8> {
    let mut hasher = ckb_hash::Blake2bBuilder::new(32)
        .personal(b"ckb-tcob-otxhash")
        .build();
    hasher.update(message.as_slice());
    let inputs_len = tx.inputs().len();
    hasher.update(&(inputs_len as u32).to_le_bytes()[..]);
    for i in 0..inputs_len {
        let input_cell = tx.inputs().get(i).unwrap();
        let input_cell_out_point = input_cell.previous_output();
        let input_cell_meta = dl.cell.get(&input_cell_out_point).unwrap();
        hasher.update(input_cell.as_slice());
        hasher.update(input_cell_meta.cell_output.as_slice());
        hasher.update(&(input_cell_meta.data_bytes as u32).to_le_bytes());
        hasher.update(&input_cell_meta.mem_cell_data.clone().unwrap());
    }
    let outputs_len = tx.outputs().len();
    hasher.update(&(outputs_len as u32).to_le_bytes()[..]);
    for i in 0..outputs_len {
        let output_cell = tx.outputs().get(i).unwrap();
        let output_cell_data: Vec<u8> = tx.outputs_data().get(i).unwrap().unpack();
        hasher.update(output_cell.as_slice());
        hasher.update(&(output_cell_data.len() as u32).to_le_bytes());
        hasher.update(output_cell_data.as_slice());
    }
    let cell_dep_len = tx.cell_deps().len();
    hasher.update(&(cell_dep_len as u32).to_le_bytes()[..]);
    for i in 0..cell_dep_len {
        let cell_dep = tx.cell_deps().get(i).unwrap();
        hasher.update(cell_dep.as_slice());
    }
    let header_dep = tx.header_deps().len();
    hasher.update(&(header_dep as u32).to_le_bytes()[..]);
    for i in 0..header_dep {
        hasher.update(tx.header_deps().get(i).unwrap().as_slice())
    }
    let mut result = vec![0u8; 32];
    hasher.finalize(&mut result);
    result
}

const ERROR_TYPESCRIPT_MISSING: i8 = 116;
const ERROR_SEAL: i8 = 117;
const ERROR_FLOW: i8 = 118;
const ERROR_WRONG_OTX: i8 = 120;
const ERROR_MOL2_UNEXPECTED: i8 = 123;

pub fn _assert_script_error(err: ckb_error::Error, err_code: i8) {
    let error_string = err.to_string();
    assert!(
        error_string.contains(format!("error code {}", err_code).as_str()),
        "error_string: {}, expected_error_code: {}",
        error_string,
        err_code
    );
}

fn generate_otx_a0(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx_builder = ckb_types::core::TransactionBuilder::default();

    // Create prior knowledge
    let prikey = "0000000000000000000000000000000000000000000000000000000000000001";
    let (prikey, args) = get_ckb_key(prikey);

    // Create cell meta
    let cell_meta_always_success = px.insert_cell_data(dl, &BINARY_ALWAYS_SUCCESS);
    let cell_meta_demo_lock = px.insert_cell_data(dl, &BINARY_TRANSACTION_COBUILD_LOCK_DEMO);
    let cell_meta_auth = px.insert_cell_data(dl, &BINARY_AUTH);
    let cell_meta_secp256k1_data = px.insert_cell_data(dl, &BINARY_SECP256K1_DATA);
    let cell_meta_i = px.insert_cell_fund(
        dl,
        px.create_script(&cell_meta_demo_lock, &args),
        None,
        &[0u8; 3000],
    );

    // Create cell dep
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_always_success));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_auth));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_secp256k1_data));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_demo_lock));
    let tx_builder = tx_builder
        .header_dep(ckb_types::packed::Byte32::from_slice(&[11u8; 32]).unwrap())
        .header_dep(ckb_types::packed::Byte32::from_slice(&[12u8; 32]).unwrap());

    // Create input
    let tx_builder = tx_builder.input(px.create_cell_input(&cell_meta_i));

    // Create output
    let tx_builder = tx_builder.output(px.create_cell_output(
        px.create_script(&cell_meta_always_success, &[]),
        Some(px.create_script(&cell_meta_always_success, &[])),
    ));

    // Create output data
    let tx_builder = tx_builder.output_data(vec![0u8; 3000].pack());

    // Create witness
    let msgs = {
        let action = schemas::basic::Action::new_builder()
            .script_info_hash(ckb_types::packed::Byte32::from_slice(&[0x00; 32]).unwrap())
            .script_hash(
                px.create_script(&cell_meta_always_success, &[])
                    .calc_script_hash(),
            )
            .data(ckb_types::bytes::Bytes::from(vec![0x42; 128]).pack())
            .build();
        let action_vec = schemas::basic::ActionVec::new_builder()
            .push(action)
            .build();
        let msgs = schemas::basic::Message::new_builder()
            .actions(action_vec)
            .build();
        msgs
    };
    let sign_msg = cobuild_create_signing_message_hash_otx(tx_builder.clone().build(), &dl, &msgs);
    println_hex("smh", &sign_msg);
    let seal = sign_pubkey_hash(prikey, &sign_msg);
    println_hex("seal", seal.as_slice());
    let seal = schemas::basic::SealPair::new_builder()
        .script_hash(
            px.create_script(&cell_meta_demo_lock, &args)
                .calc_script_hash(),
        )
        .seal(seal.pack())
        .build();
    let seal = schemas::basic::SealPairVec::new_builder()
        .push(seal)
        .build();
    let ox = schemas::basic::Otx::new_builder()
        .seals(seal)
        .message(msgs)
        .input_cells(1u32.pack())
        .output_cells(1u32.pack())
        .cell_deps(4u32.pack())
        .header_deps(2u32.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(ox)
        .build();
    let tx_builder = tx_builder.witness(wl.as_bytes().pack());

    tx_builder.build()
}

fn generate_otx_b0(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx_builder = ckb_types::core::TransactionBuilder::default();

    // Create prior knowledge
    let prikey = "0000000000000000000000000000000000000000000000000000000000000002";
    let (prikey, args) = get_ckb_key(prikey);

    // Create cell meta
    let cell_meta_always_success = px.insert_cell_data(dl, &BINARY_ALWAYS_SUCCESS);
    let cell_meta_demo_lock = px.insert_cell_data(dl, &BINARY_TRANSACTION_COBUILD_LOCK_DEMO);
    let cell_meta_auth = px.insert_cell_data(dl, &BINARY_AUTH);
    let cell_meta_secp256k1_data = px.insert_cell_data(dl, &BINARY_SECP256K1_DATA);
    let cell_meta_i = px.insert_cell_fund(
        dl,
        px.create_script(&cell_meta_demo_lock, &args),
        None,
        &[0u8; 1024],
    );

    // Create cell dep
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_always_success));
    let tx_builder: ckb_types::core::TransactionBuilder =
        tx_builder.cell_dep(px.create_cell_dep(&cell_meta_auth));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_secp256k1_data));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_demo_lock));
    let tx_builder = tx_builder
        .header_dep(ckb_types::packed::Byte32::from_slice(&[11u8; 32]).unwrap())
        .header_dep(ckb_types::packed::Byte32::from_slice(&[12u8; 32]).unwrap());

    // Create input
    let tx_builder = tx_builder.input(px.create_cell_input(&cell_meta_i));

    // Create output
    let tx_builder = tx_builder.output(px.create_cell_output(
        px.create_script(&cell_meta_always_success, &[]),
        Some(px.create_script(&cell_meta_always_success, &[])),
    ));

    // Create output data
    let tx_builder = tx_builder.output_data(vec![0u8; 1024].pack());

    // Create witness
    let msgs = {
        let action = schemas::basic::Action::new_builder()
            .script_info_hash(ckb_types::packed::Byte32::from_slice(&[0x00; 32]).unwrap())
            .script_hash(
                px.create_script(&cell_meta_always_success, &[])
                    .calc_script_hash(),
            )
            .data(ckb_types::bytes::Bytes::from(vec![0x42; 128]).pack())
            .build();
        let action_vec = schemas::basic::ActionVec::new_builder()
            .push(action)
            .build();
        let msgs = schemas::basic::Message::new_builder()
            .actions(action_vec)
            .build();
        msgs
    };
    let sign_msg = cobuild_create_signing_message_hash_otx(tx_builder.clone().build(), &dl, &msgs);
    println_hex("smh", &sign_msg);
    let seal = sign_pubkey_hash(prikey, &sign_msg);
    println_hex("seal", seal.as_slice());
    let seal = schemas::basic::SealPair::new_builder()
        .script_hash(
            px.create_script(&cell_meta_demo_lock, &args)
                .calc_script_hash(),
        )
        .seal(seal.pack())
        .build();
    let seal = schemas::basic::SealPairVec::new_builder()
        .push(seal)
        .build();
    let ox = schemas::basic::Otx::new_builder()
        .seals(seal)
        .message(msgs)
        .input_cells(1u32.pack())
        .output_cells(1u32.pack())
        .cell_deps(4u32.pack())
        .header_deps(2u32.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(ox)
        .build();
    let tx_builder = tx_builder.witness(wl.as_bytes().pack());

    tx_builder.build()
}

fn generate_otx_c0(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx_builder = ckb_types::core::TransactionBuilder::default();

    // Create prior knowledge
    let prikey = "0000000000000000000000000000000000000000000000000000000000000003";
    let (prikey, args) = get_ckb_key(prikey);

    // Create cell meta
    let cell_meta_always_success = px.insert_cell_data(dl, &BINARY_ALWAYS_SUCCESS);
    let cell_meta_demo_lock = px.insert_cell_data(dl, &BINARY_TRANSACTION_COBUILD_LOCK_DEMO);
    let cell_meta_auth = px.insert_cell_data(dl, &BINARY_AUTH);
    let cell_meta_secp256k1_data = px.insert_cell_data(dl, &BINARY_SECP256K1_DATA);
    let cell_meta_i = px.insert_cell_fund(
        dl,
        px.create_script(&cell_meta_demo_lock, &args),
        None,
        &[0u8; 4100],
    );

    // Create cell dep
    let tx_builder: ckb_types::core::TransactionBuilder =
        tx_builder.cell_dep(px.create_cell_dep(&cell_meta_always_success));
    let tx_builder: ckb_types::core::TransactionBuilder =
        tx_builder.cell_dep(px.create_cell_dep(&cell_meta_auth));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_secp256k1_data));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_demo_lock));

    // Create input
    let tx_builder = tx_builder.input(px.create_cell_input(&cell_meta_i));

    // Create output
    let tx_builder = tx_builder.output(px.create_cell_output(
        px.create_script(&cell_meta_always_success, &[]),
        Some(px.create_script(&cell_meta_always_success, &[])),
    ));

    // Create output data
    let tx_builder = tx_builder.output_data(vec![0u8; 4096].pack());

    // Create witness
    let msgs = {
        let action = schemas::basic::Action::new_builder()
            .script_info_hash(ckb_types::packed::Byte32::from_slice(&[0x00; 32]).unwrap())
            .script_hash(
                px.create_script(&cell_meta_always_success, &[])
                    .calc_script_hash(),
            )
            .data(ckb_types::bytes::Bytes::from(vec![0x42; 128]).pack())
            .build();
        let action_vec = schemas::basic::ActionVec::new_builder()
            .push(action)
            .build();
        let msgs = schemas::basic::Message::new_builder()
            .actions(action_vec)
            .build();
        msgs
    };
    let sign_msg = cobuild_create_signing_message_hash_otx(tx_builder.clone().build(), &dl, &msgs);
    println_hex("smh", &sign_msg);
    let seal = sign_pubkey_hash(prikey, &sign_msg);
    println_hex("seal", seal.as_slice());
    let seal = schemas::basic::SealPair::new_builder()
        .script_hash(
            px.create_script(&cell_meta_demo_lock, &args)
                .calc_script_hash(),
        )
        .seal(seal.pack())
        .build();
    let seal = schemas::basic::SealPairVec::new_builder()
        .push(seal)
        .build();
    let ox = schemas::basic::Otx::new_builder()
        .seals(seal)
        .message(msgs)
        .input_cells(1u32.pack())
        .output_cells(1u32.pack())
        .cell_deps(4u32.pack())
        .header_deps(0u32.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(ox)
        .build();
    let tx_builder = tx_builder.witness(wl.as_bytes().pack());

    tx_builder.build()
}

fn generate_otx_d0(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx_builder = ckb_types::core::TransactionBuilder::default();

    // Create prior knowledge
    let prikey = "0000000000000000000000000000000000000000000000000000000000000004";
    let (prikey, args) = get_ckb_key(prikey);

    // Create cell meta
    let cell_meta_always_success = px.insert_cell_data(dl, &BINARY_ALWAYS_SUCCESS);
    let cell_meta_demo_lock = px.insert_cell_data(dl, &BINARY_TRANSACTION_COBUILD_LOCK_DEMO);
    let cell_meta_auth = px.insert_cell_data(dl, &BINARY_AUTH);
    let cell_meta_secp256k1_data = px.insert_cell_data(dl, &BINARY_SECP256K1_DATA);
    let cell_meta_i =
        px.insert_cell_fund(dl, px.create_script(&cell_meta_demo_lock, &args), None, &[]);

    // Create cell dep
    let tx_builder: ckb_types::core::TransactionBuilder =
        tx_builder.cell_dep(px.create_cell_dep(&cell_meta_always_success));
    let tx_builder: ckb_types::core::TransactionBuilder =
        tx_builder.cell_dep(px.create_cell_dep(&cell_meta_auth));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_secp256k1_data));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_demo_lock));

    // Create input
    let tx_builder = tx_builder.input(px.create_cell_input(&cell_meta_i));

    // Create output
    let tx_builder = tx_builder.output(px.create_cell_output(
        px.create_script(&cell_meta_always_success, &[]),
        Some(px.create_script(&cell_meta_always_success, &[])),
    ));

    // Create output data
    let tx_builder = tx_builder.output_data(Vec::new().pack());

    // Create witness
    let msgs = {
        let action = schemas::basic::Action::new_builder()
            .script_info_hash(ckb_types::packed::Byte32::from_slice(&[0x00; 32]).unwrap())
            .script_hash(
                px.create_script(&cell_meta_always_success, &[])
                    .calc_script_hash(),
            )
            .data(ckb_types::bytes::Bytes::from(vec![0x42; 128]).pack())
            .build();
        let action_vec = schemas::basic::ActionVec::new_builder()
            .push(action)
            .build();
        let msgs = schemas::basic::Message::new_builder()
            .actions(action_vec)
            .build();
        msgs
    };
    let sign_msg = cobuild_create_signing_message_hash_otx(tx_builder.clone().build(), &dl, &msgs);
    println_hex("smh", &sign_msg);
    let seal = sign_pubkey_hash(prikey, &sign_msg);
    println_hex("seal", seal.as_slice());
    let seal = schemas::basic::SealPair::new_builder()
        .script_hash(
            px.create_script(&cell_meta_demo_lock, &args)
                .calc_script_hash(),
        )
        .seal(seal.pack())
        .build();
    let seal = schemas::basic::SealPairVec::new_builder()
        .push(seal)
        .build();
    let ox = schemas::basic::Otx::new_builder()
        .seals(seal)
        .message(msgs)
        .input_cells(1u32.pack())
        .output_cells(1u32.pack())
        .cell_deps(4u32.pack())
        .header_deps(0u32.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(ox)
        .build();
    let tx_builder = tx_builder.witness(wl.as_bytes().pack());

    tx_builder.build()
}

// Big message
fn generate_otx_e0(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx_builder = ckb_types::core::TransactionBuilder::default();

    // Create prior knowledge
    let prikey = "0000000000000000000000000000000000000000000000000000000000000004";
    let (prikey, args) = get_ckb_key(prikey);

    // Create cell meta
    let cell_meta_always_success = px.insert_cell_data(dl, &BINARY_ALWAYS_SUCCESS);
    let cell_meta_demo_lock = px.insert_cell_data(dl, &BINARY_TRANSACTION_COBUILD_LOCK_DEMO);
    let cell_meta_auth = px.insert_cell_data(dl, &BINARY_AUTH);
    let cell_meta_secp256k1_data = px.insert_cell_data(dl, &BINARY_SECP256K1_DATA);
    let cell_meta_i =
        px.insert_cell_fund(dl, px.create_script(&cell_meta_demo_lock, &args), None, &[]);

    // Create cell dep
    let tx_builder: ckb_types::core::TransactionBuilder =
        tx_builder.cell_dep(px.create_cell_dep(&cell_meta_always_success));
    let tx_builder: ckb_types::core::TransactionBuilder =
        tx_builder.cell_dep(px.create_cell_dep(&cell_meta_auth));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_secp256k1_data));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_demo_lock));

    // Create input
    let tx_builder = tx_builder.input(px.create_cell_input(&cell_meta_i));

    // Create output
    let tx_builder = tx_builder.output(px.create_cell_output(
        px.create_script(&cell_meta_always_success, &[]),
        Some(px.create_script(&cell_meta_always_success, &[])),
    ));

    // Create output data
    let tx_builder = tx_builder.output_data(Vec::new().pack());

    // Create witness
    let msgs = {
        let mut action_vec = Vec::<schemas::basic::Action>::new();
        for _ in 0..3072 {
            let action = schemas::basic::Action::new_builder()
                .script_info_hash(ckb_types::packed::Byte32::from_slice(&[0x00; 32]).unwrap())
                .script_hash(
                    px.create_script(&cell_meta_always_success, &[])
                        .calc_script_hash(),
                )
                .data(ckb_types::bytes::Bytes::from(vec![0x42; 128]).pack())
                .build();
            action_vec.push(action);
        }
        let action_vec = schemas::basic::ActionVec::new_builder()
            .extend(action_vec)
            .build();
        let msgs = schemas::basic::Message::new_builder()
            .actions(action_vec)
            .build();
        msgs
    };
    let sign_msg = cobuild_create_signing_message_hash_otx(tx_builder.clone().build(), &dl, &msgs);
    println_hex("smh", &sign_msg);
    let seal = sign_pubkey_hash(prikey, &sign_msg);
    println_hex("seal", seal.as_slice());
    let seal = schemas::basic::SealPair::new_builder()
        .script_hash(
            px.create_script(&cell_meta_demo_lock, &args)
                .calc_script_hash(),
        )
        .seal(seal.pack())
        .build();
    let seal = schemas::basic::SealPairVec::new_builder()
        .push(seal)
        .build();
    let ox = schemas::basic::Otx::new_builder()
        .seals(seal)
        .message(msgs)
        .input_cells(1u32.pack())
        .output_cells(1u32.pack())
        .cell_deps(4u32.pack())
        .header_deps(0u32.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(ox)
        .build();
    let tx_builder = tx_builder.witness(wl.as_bytes().pack());

    tx_builder.build()
}

// Failed: No seal
fn generate_otx_a1_fail(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx = generate_otx_a0(dl, px);
    let mut witnesses: Vec<ckb_types::packed::Bytes> =
        tx.witnesses().into_iter().map(|f| f).collect();

    let witness = witnesses.get(0).unwrap();
    let mut otx = None;
    let wl = schemas::top_level::WitnessLayout::from_slice(&witness.as_slice()[4..]).unwrap();
    match wl.as_reader().to_enum() {
        schemas::top_level::WitnessLayoutUnionReader::Otx(otx_reader) => {
            otx = Some(
                schemas::basic::Otx::new_unchecked(otx_reader.as_slice().to_vec().into())
                    .as_builder()
                    .seals(schemas::basic::SealPairVec::new_builder().build())
                    .build(),
            );
        }
        _ => {}
    };

    assert!(otx.is_some());
    witnesses[0] = schemas::top_level::WitnessLayout::new_builder()
        .set(otx.unwrap())
        .build()
        .as_bytes()
        .pack();

    tx.as_advanced_builder().set_witnesses(witnesses).build()
}

// Failed: The test_cobuild_otx_msg_flow is not 0
fn generate_otx_a2_fail(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx = generate_otx_a0(dl, px);
    let mut witnesses: Vec<ckb_types::packed::Bytes> =
        tx.witnesses().into_iter().map(|f| f).collect();

    let witness = witnesses.get(0).unwrap();
    let mut otx = None;

    let wl = schemas::top_level::WitnessLayout::from_slice(&witness.as_slice()[4..]).unwrap();
    match wl.as_reader().to_enum() {
        schemas::top_level::WitnessLayoutUnionReader::Otx(otx_reader) => {
            let mut seal = otx_reader
                .seals()
                .get(0)
                .unwrap()
                .seal()
                .as_slice()
                .to_vec();
            seal[0] = 0x22;

            let seal = schemas::basic::SealPair::new_unchecked(
                otx_reader
                    .seals()
                    .get(0)
                    .unwrap()
                    .as_slice()
                    .to_vec()
                    .into(),
            )
            .as_builder()
            .seal(seal.pack())
            .build();

            otx = Some(
                schemas::basic::Otx::new_unchecked(otx_reader.as_slice().to_vec().into())
                    .as_builder()
                    .seals(
                        schemas::basic::SealPairVec::new_builder()
                            .push(seal)
                            .build(),
                    )
                    .build(),
            );
        }
        _ => {}
    };

    assert!(otx.is_some());
    witnesses[0] = schemas::top_level::WitnessLayout::new_builder()
        .set(otx.unwrap())
        .build()
        .as_bytes()
        .pack();

    tx.as_advanced_builder().set_witnesses(witnesses).build()
}

// Failed: Message Action ScriptHash
fn generate_otx_a3_fail(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx = generate_otx_a0(dl, px);
    let mut witnesses: Vec<ckb_types::packed::Bytes> =
        tx.witnesses().into_iter().map(|f| f).collect();

    let witness = witnesses.get(0).unwrap();
    let mut otx = None;
    let wl = schemas::top_level::WitnessLayout::from_slice(&witness.as_slice()[4..]).unwrap();
    match wl.as_reader().to_enum() {
        schemas::top_level::WitnessLayoutUnionReader::Otx(otx_reader) => {
            let cell_meta_always_success = px.insert_cell_data(dl, &BINARY_ALWAYS_SUCCESS);
            let hash = px
                .create_script(&cell_meta_always_success, &[])
                .calc_script_hash();
            let hash = hash
                .as_builder()
                .nth0(0.into())
                .nth1(0.into())
                .nth2(0.into())
                .build();

            let msg = schemas::basic::Message::new_unchecked(
                otx_reader.message().as_slice().to_vec().into(),
            )
            .as_builder()
            .actions(
                schemas::basic::ActionVec::new_builder()
                    .push(
                        schemas::basic::Action::new_unchecked(
                            otx_reader
                                .message()
                                .actions()
                                .get(0)
                                .unwrap()
                                .as_slice()
                                .to_vec()
                                .into(),
                        )
                        .as_builder()
                        .script_hash(hash)
                        .build(),
                    )
                    .build(),
            )
            .build();

            otx = Some(
                schemas::basic::Otx::new_unchecked(otx_reader.as_slice().to_vec().into())
                    .as_builder()
                    .message(msg)
                    .build(),
            );
        }
        _ => {}
    };

    assert!(otx.is_some());
    witnesses[0] = schemas::top_level::WitnessLayout::new_builder()
        .set(otx.unwrap())
        .build()
        .as_bytes()
        .pack();

    tx.as_advanced_builder().set_witnesses(witnesses).build()
}

// Failed: The intput cells/output cells/cell deps/header deps is 0
fn generate_otx_a4_fail(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx = generate_otx_a0(dl, px);
    let mut witnesses: Vec<ckb_types::packed::Bytes> =
        tx.witnesses().into_iter().map(|f| f).collect();

    let witness = witnesses.get(0).unwrap();
    let mut otx = None;
    let wl = schemas::top_level::WitnessLayout::from_slice(&witness.as_slice()[4..]).unwrap();
    match wl.as_reader().to_enum() {
        schemas::top_level::WitnessLayoutUnionReader::Otx(otx_reader) => {
            otx = Some(
                schemas::basic::Otx::new_unchecked(otx_reader.as_slice().to_vec().into())
                    .as_builder()
                    .input_cells(0u32.pack())
                    .output_cells(0u32.pack())
                    .cell_deps(0u32.pack())
                    .header_deps(0u32.pack())
                    .build(),
            );
        }
        _ => {}
    };

    assert!(otx.is_some());
    witnesses[0] = schemas::top_level::WitnessLayout::new_builder()
        .set(otx.unwrap())
        .build()
        .as_bytes()
        .pack();

    tx.as_advanced_builder().set_witnesses(witnesses).build()
}

// Failed: None header and otx->header_deps not zero
fn generate_otx_a5_fail(dl: &mut Resource, px: &mut Pickaxer) -> ckb_types::core::TransactionView {
    let tx = generate_otx_a0(dl, px);
    tx.as_advanced_builder().set_header_deps(vec![]).build()
}

fn assemble_otx(otxs: Vec<ckb_types::core::TransactionView>) -> ckb_types::core::TransactionView {
    let tx_builder = ckb_types::core::TransactionBuilder::default();
    let os = schemas::basic::OtxStart::new_builder().build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(os)
        .build();
    let mut tx_builder = tx_builder.witness(wl.as_bytes().pack());
    for otx in otxs {
        for e in otx.cell_deps_iter() {
            tx_builder = tx_builder.cell_dep(e);
        }
        for e in otx.header_deps_iter() {
            tx_builder = tx_builder.header_dep(e);
        }
        for e in otx.inputs().into_iter() {
            tx_builder = tx_builder.input(e);
        }
        for e in otx.outputs().into_iter() {
            tx_builder = tx_builder.output(e);
        }
        for e in otx.outputs_data().into_iter() {
            tx_builder = tx_builder.output_data(e);
        }
        for e in otx.witnesses().into_iter() {
            tx_builder = tx_builder.witness(e);
        }
    }

    tx_builder.build()
}

fn merge_bytesvec<T1: IntoIterator, T2: Clone + From<<T1 as IntoIterator>::Item>>(
    v1: T1,
    v2: T1,
) -> Vec<T2> {
    let v1: Vec<T2> = v1.into_iter().map(|f| f.into()).collect();
    let v2: Vec<T2> = v2.into_iter().map(|f| f.into()).collect();
    [v1, v2].concat()
}

fn merge_tx(
    tx1: ckb_types::core::TransactionView,
    tx2: ckb_types::core::TransactionView,
) -> ckb_types::core::TransactionView {
    let tx_builder = tx1.as_advanced_builder();
    tx_builder
        .set_cell_deps(merge_bytesvec(tx1.cell_deps(), tx2.cell_deps()))
        .set_header_deps(merge_bytesvec(tx1.header_deps(), tx2.header_deps()))
        .set_inputs(merge_bytesvec(tx1.inputs(), tx2.inputs()))
        .set_outputs(merge_bytesvec(tx1.outputs(), tx2.outputs()))
        .set_witnesses(merge_bytesvec(tx1.witnesses(), tx2.witnesses()))
        .set_outputs_data(merge_bytesvec(tx1.outputs_data(), tx2.outputs_data()))
        .build()
}

#[test]
fn test_cobuild_otx_simple() {
    let mut dl = Resource::default();
    let mut px = Pickaxer::default();
    let tx = assemble_otx(vec![generate_otx_a0(&mut dl, &mut px)]);
    let tx = ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
    let verifier = Verifier::default();
    verifier.verify(&tx, &dl).unwrap();
}

#[test]
fn test_cobuild_otx_prefix() {
    let mut dl = Resource::default();
    let mut px = Pickaxer::default();
    let tx_builder = ckb_types::core::TransactionBuilder::default();

    // Create otx prefix. Add a sighash all only cell for pay fees.
    let prikey = "000000000000000000000000000000000000000000000000000000000000000f";
    let (prikey, args) = get_ckb_key(prikey);
    let cell_meta_always_success = px.insert_cell_data(&mut dl, &BINARY_ALWAYS_SUCCESS);
    let cell_meta_demo_lock = px.insert_cell_data(&mut dl, &BINARY_TRANSACTION_COBUILD_LOCK_DEMO);
    let cell_meta_i = px.insert_cell_fund(
        &mut dl,
        px.create_script(&cell_meta_demo_lock, &args),
        None,
        &[],
    );
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_always_success));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_demo_lock));
    let tx_builder = tx_builder.input(px.create_cell_input(&cell_meta_i));
    let tx_builder = tx_builder
        .output(px.create_cell_output(px.create_script(&cell_meta_always_success, &[]), None));
    let tx_builder = tx_builder.output_data(Vec::new().pack());
    let tx_builder = tx_builder.witness(vec![0x00; 102].pack());

    // Append otx
    let os = schemas::basic::OtxStart::new_builder()
        .start_cell_deps(2u32.pack())
        .start_header_deps(0u32.pack())
        .start_input_cell(1u32.pack())
        .start_output_cell(1u32.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(os)
        .build();
    let mut tx_builder = tx_builder.witness(wl.as_bytes().pack());
    let otxs = vec![
        generate_otx_a0(&mut dl, &mut px),
        generate_otx_b0(&mut dl, &mut px),
    ];
    for otx in otxs {
        for e in otx.cell_deps_iter() {
            tx_builder = tx_builder.cell_dep(e);
        }
        for e in otx.header_deps_iter() {
            tx_builder = tx_builder.header_dep(e);
        }
        for e in otx.inputs().into_iter() {
            tx_builder = tx_builder.input(e);
        }
        for e in otx.outputs().into_iter() {
            tx_builder = tx_builder.output(e);
        }
        for e in otx.outputs_data().into_iter() {
            tx_builder = tx_builder.output_data(e);
        }
        for e in otx.witnesses().into_iter() {
            tx_builder = tx_builder.witness(e);
        }
    }

    // Create sign for prefix
    let sign_message =
        cobuild_create_signing_message_hash_sighash_all_only(tx_builder.clone().build(), &dl);
    println_hex("smh", &sign_message);

    let seal = sign_pubkey_hash(prikey, &sign_message);
    println_hex("seal", seal.as_slice());
    let so = schemas::basic::SighashAllOnly::new_builder()
        .seal(seal.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(so)
        .build();
    // assert_eq!(wl.as_bytes().pack().len(), 102);
    let mut wb = tx_builder.clone().build().witnesses().as_builder();
    wb.replace(0, wl.as_bytes().pack());
    let tx_builder = tx_builder.set_witnesses(wb.build().into_iter().collect());

    let tx = tx_builder.build();
    let tx = ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
    let verifier = Verifier::default();
    verifier.verify(&tx, &dl).unwrap();
}

#[test]
fn test_cobuild_otx_prefix_and_suffix() {
    let mut dl = Resource::default();
    let mut px = Pickaxer::default();
    let tx_builder = ckb_types::core::TransactionBuilder::default();

    // Create otx prefix
    let cell_meta_always_success = px.insert_cell_data(&mut dl, &BINARY_ALWAYS_SUCCESS);
    let cell_meta_i = px.insert_cell_fund(
        &mut dl,
        px.create_script(&cell_meta_always_success, &[]),
        None,
        &[],
    );
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_always_success));
    let tx_builder = tx_builder.input(px.create_cell_input(&cell_meta_i));
    let tx_builder = tx_builder.output(px.create_cell_output(
        px.create_script(&cell_meta_always_success, &[]),
        Some(px.create_script(&cell_meta_always_success, &[])),
    ));
    let tx_builder = tx_builder.output_data(vec![].pack());

    // Append otx
    let os = schemas::basic::OtxStart::new_builder()
        .start_cell_deps(1u32.pack())
        .start_header_deps(0u32.pack())
        .start_input_cell(1u32.pack())
        .start_output_cell(1u32.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(os)
        .build();
    let mut tx_builder = tx_builder.witness(wl.as_bytes().pack());
    let otxs = vec![
        generate_otx_a0(&mut dl, &mut px),
        generate_otx_b0(&mut dl, &mut px),
    ];
    for otx in otxs {
        for e in otx.cell_deps_iter() {
            tx_builder = tx_builder.cell_dep(e);
        }
        for e in otx.header_deps_iter() {
            tx_builder = tx_builder.header_dep(e);
        }
        for e in otx.inputs().into_iter() {
            tx_builder = tx_builder.input(e);
        }
        for e in otx.outputs().into_iter() {
            tx_builder = tx_builder.output(e);
        }
        for e in otx.outputs_data().into_iter() {
            tx_builder = tx_builder.output_data(e);
        }
        for e in otx.witnesses().into_iter() {
            tx_builder = tx_builder.witness(e);
        }
    }

    // Create otx suffix. Add a sighash all only cell for pay fees.
    let prikey = "000000000000000000000000000000000000000000000000000000000000000f";
    let (prikey, args) = get_ckb_key(prikey);
    let cell_meta_always_success = px.insert_cell_data(&mut dl, &BINARY_ALWAYS_SUCCESS);
    let cell_meta_demo_lock = px.insert_cell_data(&mut dl, &BINARY_TRANSACTION_COBUILD_LOCK_DEMO);
    let cell_meta_i = px.insert_cell_fund(
        &mut dl,
        px.create_script(&cell_meta_demo_lock, &args),
        None,
        &[],
    );
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_always_success));
    let tx_builder = tx_builder.cell_dep(px.create_cell_dep(&cell_meta_demo_lock));
    let tx_builder = tx_builder.input(px.create_cell_input(&cell_meta_i));
    let tx_builder = tx_builder
        .output(px.create_cell_output(px.create_script(&cell_meta_always_success, &[]), None));
    let tx_builder = tx_builder.output_data(Vec::new().pack());
    let sign =
        cobuild_create_signing_message_hash_sighash_all_only(tx_builder.clone().build(), &dl);
    let seal = sign_pubkey_hash(prikey, &sign);
    println_hex("seal", seal.as_slice());
    let so = schemas::basic::SighashAllOnly::new_builder()
        .seal(seal.pack())
        .build();
    let wl = schemas::top_level::WitnessLayout::new_builder()
        .set(so)
        .build();
    let tx_builder = tx_builder.witness(wl.as_bytes().pack());

    let tx = tx_builder.build();
    let tx = ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
    let verifier = Verifier::default();
    verifier.verify(&tx, &dl).unwrap();
}

#[test]
fn test_cobuild_otx_same_lock_script() {
    let mut dl = Resource::default();
    let mut px = Pickaxer::default();
    let tx = assemble_otx(vec![
        generate_otx_a0(&mut dl, &mut px),
        generate_otx_a0(&mut dl, &mut px),
    ]);
    let tx = ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
    let verifier = Verifier::default();
    verifier.verify(&tx, &dl).unwrap();
}

#[test]
fn test_cobuild_otx_random() {
    type Fntype = dyn Fn(&mut Resource, &mut Pickaxer) -> ckb_types::core::TransactionView;
    let mut rgen = rand::prelude::thread_rng();
    let mut success_set = Vec::<(&str, Box<Fntype>)>::new();
    success_set.push(("a0", Box::new(generate_otx_a0)));
    success_set.push(("b0", Box::new(generate_otx_b0)));
    success_set.push(("c0", Box::new(generate_otx_c0)));
    success_set.push(("d0", Box::new(generate_otx_d0)));
    for i in 0..success_set.len() {
        let mut dl = Resource::default();
        let mut px = Pickaxer::default();
        println_log(format!("case: {}", success_set[i].0).as_str());
        let tx = assemble_otx(vec![success_set[i].1(&mut dl, &mut px)]);
        let tx =
            ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
        let verifier = Verifier::default();
        verifier.verify(&tx, &dl).unwrap();
    }
    for _ in 0..32 {
        let mut dl = Resource::default();
        let mut px = Pickaxer::default();
        let mut hint = vec![];
        let mut data = vec![];
        for _ in 0..2 + (rgen.next_u32() as usize % 3) {
            let nf = success_set.choose(&mut rgen).unwrap();
            hint.push(nf.0);
            data.push(nf.1(&mut dl, &mut px));
        }
        println_log(format!("case: {}", hint.join(" + ")).as_str());
        let tx = assemble_otx(data);
        let tx =
            ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
        let verifier = Verifier::default();
        verifier.verify(&tx, &dl).unwrap();
    }

    // Failed
    // Compared with success_set, Error code is added
    let mut failed_set = Vec::<(&str, Box<Fntype>, i8)>::new();
    failed_set.push(("a1", Box::new(generate_otx_a1_fail), ERROR_SEAL));
    failed_set.push(("a2", Box::new(generate_otx_a2_fail), ERROR_FLOW));
    failed_set.push((
        "a3",
        Box::new(generate_otx_a3_fail),
        ERROR_TYPESCRIPT_MISSING,
    ));
    failed_set.push(("a4", Box::new(generate_otx_a4_fail), ERROR_WRONG_OTX));
    failed_set.push(("a5", Box::new(generate_otx_a5_fail), ERROR_MOL2_UNEXPECTED));

    for i in 0..failed_set.len() {
        let mut dl = Resource::default();
        let mut px = Pickaxer::default();
        println_log(format!("case: {}", failed_set[i].0).as_str());
        let tx = assemble_otx(vec![failed_set[i].1(&mut dl, &mut px)]);
        let tx =
            ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
        let verifier = Verifier::default();
        verifier.verify(&tx, &dl).unwrap_err();
        // assert_script_error(verifier.verify(&tx, &dl).unwrap_err(), failed_set[i].2);
    }

    // n success + 1 failed
    for _ in 0..32 {
        let mut dl = Resource::default();
        let mut px = Pickaxer::default();
        let mut hint = vec![];
        let mut data = vec![];
        for _ in 0..2 + (rgen.next_u32() as usize % 3) {
            let nf = success_set.choose(&mut rgen).unwrap();
            hint.push(nf.0);
            data.push(nf.1(&mut dl, &mut px));
        }

        let nf = failed_set.choose(&mut rgen).unwrap();
        hint.push(nf.0);
        data.push(nf.1(&mut dl, &mut px));

        // println_log(format!("case: {}", hint.join(" + ")).as_str());
        let tx = assemble_otx(data);
        let tx =
            ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
        let verifier = Verifier::default();
        verifier.verify(&tx, &dl).unwrap_err();
        // assert_script_error(verifier.verify(&tx, &dl).unwrap_err(), nf.2);
    }

    // n success + n failed.
    //  unknow error code
    for _ in 0..32 {
        let mut dl = Resource::default();
        let mut px = Pickaxer::default();
        let mut hint = vec![];
        let mut data = vec![];
        for _ in 0..2 + (rgen.next_u32() as usize % 3) {
            let nf = success_set.choose(&mut rgen).unwrap();
            hint.push(nf.0);
            data.push(nf.1(&mut dl, &mut px));
        }

        for _ in 0..2 + (rgen.next_u32() as usize % 3) {
            let nf = failed_set.choose(&mut rgen).unwrap();
            hint.push(nf.0);
            data.push(nf.1(&mut dl, &mut px));
        }

        // println_log(format!("case: {}", hint.join(" + ")).as_str());
        let tx = assemble_otx(data);
        let tx =
            ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
        let verifier = Verifier::default();
        let _error_code = verifier.verify(&tx, &dl).unwrap_err();
        // println!("random multi failed, error code: {}", error_code);
    }
}

#[test]
fn test_cobuild_otx_double_otx_start() {
    let mut dl = Resource::default();
    let mut px = Pickaxer::default();

    let tx = assemble_otx(vec![generate_otx_a0(&mut dl, &mut px)]);

    let tx = {
        let mut witnesses: Vec<ckb_types::packed::Bytes> =
            tx.witnesses().into_iter().map(|f| f).collect();

        let os = schemas::basic::OtxStart::new_builder().build();
        let wl = schemas::top_level::WitnessLayout::new_builder()
            .set(os)
            .build();
        witnesses.insert(0, wl.as_bytes().pack());

        tx.as_advanced_builder().set_witnesses(witnesses).build()
    };

    let tx = ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
    let verifier = Verifier::default();
    verifier.verify(&tx, &dl).unwrap_err();
}

#[test]
fn test_cobuild_otx_noexistent_otx_id() {
    let mut dl = Resource::default();
    let mut px = Pickaxer::default();

    let tx = assemble_otx(vec![generate_otx_a0(&mut dl, &mut px)]);
    let tx = {
        let mut witnesses: Vec<ckb_types::packed::Bytes> =
            tx.witnesses().into_iter().map(|f| f).collect();

        let os = schemas::basic::OtxStart::new_builder().build();
        let wl = schemas::top_level::WitnessLayout::new_builder()
            .set(os)
            .build();
        witnesses.insert(0, wl.as_bytes().pack());

        tx.as_advanced_builder().set_witnesses(witnesses).build()
    };

    let mut witnesses: Vec<ckb_types::packed::Bytes> =
        tx.witnesses().into_iter().map(|f| f).collect();
    let mut witness = witnesses.get(1).unwrap().as_slice().to_vec();
    witness[4..8].copy_from_slice(&(4278190084u32 + 2).to_le_bytes()); // WitnessLayoutOtxStart + 1
    witnesses[1] = witness.pack();

    let tx = tx.as_advanced_builder().set_witnesses(witnesses).build();

    let tx = ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
    let verifier = Verifier::default();
    let result_verifier = verifier.verify(&tx, &dl);
    result_verifier.unwrap_err();
}

#[test]
fn test_cobuild_otx_double_input() {
    let mut dl = Resource::default();
    let mut px = Pickaxer::default();

    let tx = merge_tx(
        assemble_otx(vec![generate_otx_a0(&mut dl, &mut px)]),
        assemble_otx(vec![generate_otx_a0(&mut dl, &mut px)]),
    );

    let tx = ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();
    let verifier = Verifier::default();
    verifier.verify(&tx, &dl).unwrap_err();
}

#[test]
fn test_cobuild_big_message() {
    let mut dl = Resource::default();
    let mut px = Pickaxer::default();
    let tx = assemble_otx(vec![generate_otx_e0(&mut dl, &mut px)]);
    let tx = ckb_types::core::cell::resolve_transaction(tx, &mut HashSet::new(), &dl, &dl).unwrap();

    let verifier = Verifier::default();
    verifier.verify(&tx, &dl).unwrap();
}
