use super::tx::*;
use ckb_testtool::{ckb_error::Error, bytes::Bytes, ckb_types::prelude::*};
use ckb_transaction_cobuild::schemas::basic::{Message, Action, ActionVec};
use molecule::prelude::*;

const MAX_CYCLES: u64 = 10_000_000;

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
    let others_witnesses = vec![];

    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], others_witnesses);
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
fn test_success_sighash_all_only() {
    let others_witnesses = vec![];

    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], others_witnesses);

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
    let others_witnesses = vec![];

    let mut witnesses = MessageWitnesses::new(vec![3, 1, 2], others_witnesses);
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
    let actions = vec![
        Action::new_builder()
            .script_hash(type_script.calc_script_hash())
            .data(Bytes::from(b"MINT".to_vec()).pack())
            .build(),
    ];
    let msg = Message::new_builder().actions(ActionVec::new_builder().set(actions).build());
    witnesses.message_data.get_mut(0).unwrap().action = Some(msg.build());

    let tx = sign_tx(&mut witnesses, tx, resolved_inputs);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
