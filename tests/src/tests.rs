use super::tx::*;
use ckb_testtool::ckb_error::Error;
use ckb_testtool::ckb_types::prelude::*;
use ckb_typed_message::schemas::{
    basic::{Otx, OtxStart},
    top_level::{ExtendedWitness, ExtendedWitnessUnion},
};

const MAX_CYCLES: u64 = 10_000_000;

// error numbers
fn _assert_script_error(err: Error, err_code: i8) {
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
    let others_witnesses = vec![
        ExtendedWitness::new_builder()
            .set(ExtendedWitnessUnion::OtxStart(
                OtxStart::new_builder().build().into(),
            ))
            .build(),
        ExtendedWitness::new_builder()
            .set(ExtendedWitnessUnion::Otx(Otx::new_builder().build().into()))
            .build(),
    ];

    let mut witnesses = TypedMsgWitnesses::new(2, others_witnesses);
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
