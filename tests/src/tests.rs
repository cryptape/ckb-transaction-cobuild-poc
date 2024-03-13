use super::{tx::*, *};
use ckb_testtool::ckb_crypto::secp::{Generator, Message as SecpMessage};
use ckb_testtool::{
    bytes::Bytes,
    ckb_error::Error,
    ckb_hash::blake2b_256,
    ckb_types::{
        core::{DepType, TransactionBuilder},
        packed::*,
        prelude::*,
    },
    context::Context,
};
use ckb_transaction_cobuild::blake2b::new_otx_blake2b;
use ckb_transaction_cobuild::schemas::{
    basic::{
        Action, ActionVec, Message, Otx, OtxStart, ResolvedInputs, SealPair, SealPairVec,
        SighashAll, SighashAllOnly,
    },
    top_level::{WitnessLayout, WitnessLayoutUnion},
};
use molecule::prelude::*;

const MAX_CYCLES: u64 = 20_000_000;

// error numbers
fn assert_script_error(err: Error, err_code: i8) {
    let error_string = err.to_string();
    assert!(
        error_string.contains(format!("error code {} ", err_code).as_str()),
        "error_string: {}, expected_error_code: {}",
        error_string,
        err_code
    );
}

#[test]
fn test_success_sighash_all() {
    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], vec![]);
    witnesses.set_with_action(1);

    // deploy contract
    let (tx, resolved_inputs, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_success_with_other_witness_than_4() {
    let others_witnesses = vec![Bytes::from([00, 00, 00, 00].to_vec())];

    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], others_witnesses);
    witnesses.set_with_action(1);

    // deploy contract
    let (tx, resolved_inputs, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let _cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_success_with_other_empty_witness() {
    let others_witnesses = vec![Bytes::new()];

    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], others_witnesses);
    witnesses.set_with_action(1);

    // deploy contract
    let (tx, resolved_inputs, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let _cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_success_with_other_witnesslayout() {
    let others_witnesses = vec![WitnessLayout::new_builder()
        .set(WitnessLayoutUnion::SighashAllOnly(
            SighashAllOnly::new_builder()
                .seal(Bytes::from_static(&[0u8; 64]).pack())
                .build(),
        ))
        .build()
        .as_bytes()];

    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], others_witnesses);
    witnesses.set_with_action(1);

    // deploy contract
    let (tx, resolved_inputs, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let _cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
}

#[test]
fn test_failed_dup_sighashall() {
    let others_witnesses = vec![
        WitnessLayout::new_builder()
            .set(WitnessLayoutUnion::SighashAllOnly(
                SighashAllOnly::new_builder()
                    .seal(Bytes::from_static(&[0u8; 64]).pack())
                    .build(),
            ))
            .build()
            .as_bytes(),
        WitnessLayout::new_builder()
            .set(WitnessLayoutUnion::SighashAll(
                SighashAll::new_builder()
                    .message(
                        Message::new_builder()
                            .actions(
                                ActionVec::new_builder()
                                    .push(
                                        Action::new_builder()
                                            .script_hash(Byte32::new([0u8; 32]))
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .seal(Bytes::from_static(&[0u8; 64]).pack())
                    .build(),
            ))
            .build()
            .as_bytes(),
    ];

    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], others_witnesses);
    witnesses.set_with_action(1);

    // deploy contract
    let (tx, resolved_inputs, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect_err("pass verification");
    assert_script_error(err, 7); // return Error::WrongWitnessLayout
}

#[test]
fn test_success_sighash_all_only() {
    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], vec![]);

    // deploy contract
    let (tx, resolved_inputs, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_failed_pubkey() {
    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], vec![]);
    witnesses.set_with_action(1);
    witnesses.message_data[2].config_failed_pubkey_hash = true;

    witnesses.update();

    // deploy contract
    let (tx, resolved_inputs, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect_err("pass verification");
    assert_script_error(err, 5); // return Error::AuthError
}

#[test]
fn test_type_script() {
    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], vec![]);
    let (tx, resolved_inputs, context) = gen_tx(&witnesses);
    let type_script = tx.outputs().get(1).unwrap().type_().to_opt().unwrap();
    let actions = vec![Action::new_builder()
        .script_hash(type_script.calc_script_hash())
        .data(Bytes::from(b"MINT".to_vec()).pack())
        .build()];
    let msg = Message::new_builder().actions(ActionVec::new_builder().set(actions).build());
    witnesses.message_data.get_mut(0).unwrap().action = Some(msg.build());

    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_success_otx() {
    // deploy contract
    let mut context = Context::default();
    let loader = Loader::default();
    let otx_bin = loader.load_binary("transaction-cobuild-otx-lock-demo");
    let auth_bin = loader.load_binary("../auth");
    let secp256k1_bin = loader.load_binary("../secp256k1_data_20210801");

    let otx_out_point = context.deploy_cell(otx_bin);
    let auth_out_point = context.deploy_cell(auth_bin);
    let secp256k1_out_point = context.deploy_cell(secp256k1_bin);

    // prepare scripts
    let privkey = Generator::random_privkey();
    let pubkey_hash: [u8; 20] = blake2b_256(privkey.pubkey().unwrap().serialize().as_slice())[..20]
        .try_into()
        .unwrap();

    let lock_script = context
        .build_script(&otx_out_point, pubkey_hash.to_vec().into())
        .expect("script");

    // prepare cells
    let input_cell = CellOutput::new_builder()
        .capacity(1000u64.pack())
        .lock(lock_script.clone())
        .build();
    let input_out_point = context.create_cell(input_cell.clone(), Default::default());
    let resolved_inputs = ResolvedInputs::new_builder()
        .outputs(CellOutputVec::new_builder().push(input_cell).build())
        .outputs_data(BytesVec::new_builder().push(Default::default()).build())
        .build();

    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
    ];
    let data: Bytes = vec![0u8; 5000].into();
    let outputs_data = vec![data, Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .cell_dep(
            CellDep::new_builder()
                .out_point(auth_out_point)
                .dep_type(DepType::Code.into())
                .build(),
        )
        .cell_dep(
            CellDep::new_builder()
                .out_point(secp256k1_out_point)
                .dep_type(DepType::Code.into())
                .build(),
        )
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = context.complete_tx(tx);

    // sign otx
    let message = Message::new_builder().build();
    let otx_signing_message_hash =
        generate_otx_signing_message_hash(&message, &tx.data().raw(), &resolved_inputs);
    let signature = privkey
        .sign_recoverable(&SecpMessage::from_slice(&otx_signing_message_hash).unwrap())
        .unwrap()
        .serialize();
    let seal_pair = SealPair::new_builder()
        .script_hash(lock_script.calc_script_hash())
        .seal(Bytes::from(signature.to_vec()).pack())
        .build();

    let otx_start = OtxStart::new_builder()
        .start_cell_deps(0u32.pack())
        .start_header_deps(0u32.pack())
        .start_input_cell(0u32.pack())
        .start_output_cell(0u32.pack())
        .build();
    let witness1 = WitnessLayout::new_builder()
        .set(WitnessLayoutUnion::OtxStart(otx_start))
        .build()
        .as_bytes()
        .pack();

    let otx = Otx::new_builder()
        .cell_deps(0u32.pack())
        .header_deps(0u32.pack())
        .input_cells(1u32.pack())
        .output_cells(2u32.pack())
        .message(message)
        .seals(SealPairVec::new_builder().push(seal_pair).build())
        .build();
    let witness2 = WitnessLayout::new_builder()
        .set(WitnessLayoutUnion::Otx(otx))
        .build()
        .as_bytes()
        .pack();

    // run
    let tx = tx
        .as_advanced_builder()
        .set_witnesses(vec![witness1, witness2])
        .build();
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

fn generate_otx_signing_message_hash(
    message: &Message,
    otx: &RawTransaction,
    resolved_inputs: &ResolvedInputs,
) -> [u8; 32] {
    let mut hasher = new_otx_blake2b();
    // message
    hasher.update(message.as_slice());
    // otx inputs
    let inputs_len = otx.inputs().len();
    debug_assert!(inputs_len == resolved_inputs.outputs().len());
    debug_assert!(inputs_len == resolved_inputs.outputs_data().len());
    hasher.update(&(inputs_len as u32).to_le_bytes());
    for i in 0..inputs_len {
        hasher.update(otx.inputs().get(i).unwrap().as_slice());
        let input_cell = resolved_inputs.outputs().get(i).unwrap();
        hasher.update(&input_cell.as_slice());
        let input_cell_data = resolved_inputs.outputs_data().get(i).unwrap();
        hasher.update(&(input_cell_data.len() as u32).to_le_bytes());
        hasher.update(&input_cell_data.raw_data());
    }
    // otx outputs
    let outputs_len = otx.outputs().len();
    debug_assert!(outputs_len == otx.outputs_data().len());
    hasher.update(&(outputs_len as u32).to_le_bytes());
    for i in 0..outputs_len {
        hasher.update(otx.outputs().get(i).unwrap().as_slice());
        hasher.update(otx.outputs_data().get(i).unwrap().as_slice());
    }
    // otx deps (in this unit test, we don't sign any cell and header deps for otx)
    hasher.update(&0u32.to_le_bytes());
    hasher.update(&0u32.to_le_bytes());

    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    result
}
