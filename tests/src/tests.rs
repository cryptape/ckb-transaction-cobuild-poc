use super::tx::*;
use ckb_testtool::ckb_error::Error;

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
fn test_success() {
    let others_witnesses = vec![];

    let mut witnesses = TypedMsgWitnesses::new(vec![3, 1, 2], others_witnesses);
    witnesses.set_with_action(1);

    // deploy contract
    let (tx, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_failed_pubkey() {
    let others_witnesses = vec![];

    let mut witnesses = TypedMsgWitnesses::new(vec![3, 1, 2], others_witnesses);
    witnesses.set_with_action(1);
    witnesses.typed_msg_datas[2].config_failed_pubkey_hash = true;
    
    witnesses.update();

    // deploy contract
    let (tx, context) = gen_tx(&witnesses);
    let tx = sign_tx(&mut witnesses, tx);
    // run
    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect_err("pass verification");
    assert_script_error(err, 5); // return Error::AuthError
}
