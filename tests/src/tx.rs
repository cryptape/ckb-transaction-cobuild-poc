use super::*;
use ckb_testtool::ckb_crypto::secp::{Generator, Message, Privkey};
use ckb_testtool::ckb_hash::new_blake2b;
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{DepType, TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_testtool::context::Context;
use ckb_typed_message::schemas::{
    basic::{Sighash, SighashWithAction, TypedMessage},
    blockchain,
    // blockchain::{BytesOpt, WitnessArgs},
    top_level::{ExtendedWitness, ExtendedWitnessUnion},
};
use molecule::prelude::*;
use rand::{thread_rng, RngCore};

pub struct TypedMsgData {
    pub privkey: Privkey,
    pub action: Option<TypedMessage>,
    pub sign: Option<Vec<u8>>,
}
impl TypedMsgData {
    pub fn new() -> Self {
        Self {
            privkey: Generator::random_privkey(),
            action: None,
            sign: None,
        }
    }

    pub fn new_extended_witness(&self) -> ExtendedWitness {
        let sign = match &self.sign {
            Some(v) => v.clone(),
            None => [0u8; 65].to_vec(),
        };

        match &self.action {
            Some(action) => ExtendedWitness::new_builder()
                .set(ExtendedWitnessUnion::SighashWithAction(
                    SighashWithAction::new_builder()
                        .lock(
                            blockchain::Bytes::new_builder()
                                .set(sign.iter().map(|f| f.clone().into()).collect())
                                .build(),
                        )
                        .message(action.clone())
                        .build()
                        .into(),
                ))
                .build(),
            None => ExtendedWitness::new_builder()
                .set(ExtendedWitnessUnion::Sighash(
                    Sighash::new_builder()
                        .lock(
                            blockchain::Bytes::new_builder()
                                .set(sign.iter().map(|f| f.clone().into()).collect())
                                .build(),
                        )
                        .build()
                        .into(),
                ))
                .build(),
        }
    }

    pub fn get_pubkey_hash(&self) -> [u8; 20] {
        let pub_key = self.privkey.pubkey().expect("pubkey").serialize();
        let pub_hash = ckb_testtool::ckb_hash::blake2b_256(pub_key.as_slice());
        pub_hash[0..20].try_into().unwrap()
    }
}

pub struct TypedMsgWitnesses {
    pub typed_msg_datas: Vec<TypedMsgData>,
    pub others: Vec<ExtendedWitness>,
}
impl TypedMsgWitnesses {
    pub fn new(num: usize, others: Vec<ExtendedWitness>) -> Self {
        let mut typed_msg_datas = Vec::new();
        for _i in 0..num {
            typed_msg_datas.push(TypedMsgData::new());
        }

        Self {
            typed_msg_datas,
            others,
        }
    }

    pub fn set_with_action(&mut self, index: usize) {
        use ckb_typed_message::schemas::basic::{Action, ScriptAction, TypedMessageV1};

        let script_actions = vec![
            ScriptAction::new_builder()
                .script_hash(Self::rng_byte32())
                .action(
                    Action::new_builder()
                        .info_hash(Self::rng_byte32())
                        .data(Self::rng_bytes(30))
                        .build(),
                )
                .build(),
            ScriptAction::new_builder()
                .script_hash(Self::rng_byte32())
                .build(),
        ];

        let msg = TypedMessage::new_builder()
            .set(TypedMessageV1::new_builder().set(script_actions).build());

        self.typed_msg_datas.get_mut(index).unwrap().action = Some(msg.build());
    }

    pub fn get_witnesses(&self) -> Vec<Bytes> {
        let mut witnesses = Vec::new();

        for data in &self.typed_msg_datas {
            let d = data.new_extended_witness();
            witnesses.push(d.as_bytes());
        }

        for w in &self.others {
            witnesses.push(w.as_bytes());
        }

        witnesses
    }

    pub fn get_action(&self) -> &TypedMessage {
        for d in &self.typed_msg_datas {
            if d.action.is_some() {
                return d.action.as_ref().unwrap();
            }
        }
        panic!("none")
    }

    pub fn rng_byte32() -> blockchain::Byte32 {
        let mut buf = [0u8; 32];
        thread_rng().fill_bytes(&mut buf);
        let buf: Vec<Byte> = buf.iter().map(|f| f.clone().into()).collect();

        blockchain::Byte32::new_builder()
            .set(buf.try_into().unwrap())
            .build()
    }

    pub fn rng_bytes(len: usize) -> blockchain::Bytes {
        let mut buf = Vec::with_capacity(len);
        buf.resize(len, 0);

        thread_rng().fill_bytes(&mut buf);

        blockchain::Bytes::new_builder()
            .set(buf.iter().map(|f| f.clone().into()).collect())
            .build()
    }
}

fn append_cells(context: &mut Context) -> (OutPoint, TransactionBuilder) {
    let loader = Loader::default();
    let tx = TransactionBuilder::default();

    let tx = tx
        .cell_dep(
            CellDepBuilder::default()
                .out_point(context.deploy_cell(loader.load_binary("../auth")))
                .dep_type(DepType::Code.into())
                .build(),
        )
        .cell_dep(
            CellDepBuilder::default()
                .out_point(context.deploy_cell(loader.load_binary("../secp256k1_data_20210801")))
                .dep_type(DepType::Code.into())
                .build(),
        );

    (
        context.deploy_cell(loader.load_binary("typed-message-lock-demo")),
        tx,
    )
}

pub fn gen_tx(witnesses: &TypedMsgWitnesses) -> (TransactionView, Context) {
    let mut context = Context::default();
    let (out_point, mut tx) = append_cells(&mut context);

    for data in &witnesses.typed_msg_datas {
        let lock_script = context
            .build_script(&out_point, Bytes::from(data.get_pubkey_hash().to_vec()))
            .expect("script");
        let input_out_point = context.create_cell(
            CellOutput::new_builder()
                .capacity(1000u64.pack())
                .lock(lock_script.clone())
                .build(),
            Bytes::new(),
        );
        let input = CellInput::new_builder()
            .previous_output(input_out_point)
            .build();

        tx = tx.input(input)
    }

    let output_lock_script = context
        .build_script(&out_point, Bytes::from(vec![]))
        .expect("script");

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(output_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(output_lock_script.clone())
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    let witnesses: Vec<Bytes> = witnesses.get_witnesses();

    // build transaction
    let tx = tx
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .set_witnesses(witnesses.iter().map(|f| f.pack()).collect())
        .build();
    let tx = context.complete_tx(tx);
    (tx, context)
}

fn generate_skeleton_hash(tx: &TransactionView) -> [u8; 32] {
    let mut hasher = new_blake2b();

    hasher.update(tx.hash().as_slice());

    for i in tx.inputs().len()..tx.witnesses().len() {
        let w = tx.witnesses().get(i).unwrap();
        let w = w.as_slice()[4..].to_vec();

        hasher.update(&w.len().to_le_bytes());
        hasher.update(&w);
    }

    let mut ret_hash = [0u8; 32];
    hasher.finalize(&mut ret_hash);
    ret_hash
}

pub fn sign_tx(witnesses: &mut TypedMsgWitnesses, tx: TransactionView) -> TransactionView {
    // get sign message
    let skeleton_hash = generate_skeleton_hash(&tx);

    for i in 0..tx.inputs().len() {
        let typed_msg = witnesses.get_action().as_slice().to_vec();

        let mut hasher = new_blake2b();
        hasher.update(&skeleton_hash);
        hasher.update(&typed_msg.len().to_le_bytes());
        hasher.update(&typed_msg);

        let mut digest_message = [0u8; 32];
        hasher.finalize(&mut digest_message);

        let sign = witnesses
            .typed_msg_datas
            .get(i)
            .unwrap()
            .privkey
            .sign_recoverable(&Message::from_slice(&digest_message).unwrap())
            .expect("sign")
            .serialize();

        witnesses.typed_msg_datas.get_mut(i).unwrap().sign = Some(sign);
    }

    let witnesses = witnesses.get_witnesses();
    tx.as_advanced_builder()
        .set_witnesses(witnesses.iter().map(|f| f.pack()).collect())
        .build()
}
