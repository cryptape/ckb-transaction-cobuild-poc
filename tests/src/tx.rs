use super::*;
use ckb_testtool::ckb_crypto::secp::{Generator, Message as SecpMessage, Privkey};
use ckb_testtool::ckb_hash::new_blake2b;
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{DepType, TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_testtool::context::Context;
use ckb_transaction_cobuild::schemas::{
    basic::{Message, SighashAll, SighashAllOnly},
    blockchain,
    top_level::{WitnessLayout, WitnessLayoutUnion},
};
use molecule::prelude::*;
use rand::{thread_rng, RngCore};

pub struct MessageData {
    pub privkey: Privkey,
    pub pubkey_hash: [u8; 20],
    pub group_size: usize,
    pub action: Option<Message>,
    pub sign: Option<Vec<u8>>,

    pub config_failed_pubkey_hash: bool,
}
impl MessageData {
    pub fn new(group_size: usize) -> Self {
        let privkey = Generator::random_privkey();
        let pubkey_hash = {
            let pub_hash = ckb_testtool::ckb_hash::blake2b_256(
                privkey.pubkey().expect("pubkey").serialize().as_slice(),
            );
            pub_hash[..20].try_into().unwrap()
        };

        Self {
            privkey,
            pubkey_hash,
            group_size,
            action: None,
            sign: None,

            config_failed_pubkey_hash: false,
        }
    }

    pub fn new_extended_witness(&self) -> WitnessLayout {
        let sign = match &self.sign {
            Some(v) => v.clone(),
            None => [0u8; 65].to_vec(),
        };

        match &self.action {
            Some(action) => WitnessLayout::new_builder()
                .set(WitnessLayoutUnion::SighashAll(
                    SighashAll::new_builder()
                        .seal(
                            blockchain::Bytes::new_builder()
                                .set(sign.iter().map(|f| f.clone().into()).collect())
                                .build(),
                        )
                        .message(action.clone())
                        .build()
                        .into(),
                ))
                .build(),
            None => WitnessLayout::new_builder()
                .set(WitnessLayoutUnion::SighashAllOnly(
                    SighashAllOnly::new_builder()
                        .seal(
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

    pub fn update_config(&mut self) {
        if self.config_failed_pubkey_hash {
            let mut buf = [0u8; 20];
            thread_rng().fill_bytes(&mut buf);
            self.pubkey_hash = buf;
        }
    }
}

pub struct MessageWitnesses {
    pub message_data: Vec<MessageData>,
    pub others: Vec<WitnessLayout>,
}

impl MessageWitnesses {
    pub fn new(groups_size: Vec<usize>, others: Vec<WitnessLayout>) -> Self {
        let mut message_data = Vec::new();
        for group_size in groups_size {
            message_data.push(MessageData::new(group_size));
        }

        Self {
            message_data,
            others,
        }
    }

    pub fn update(&mut self) {
        for d in &mut self.message_data {
            d.update_config();
        }
    }

    pub fn set_with_action(&mut self, index: usize) {
        use ckb_transaction_cobuild::schemas::basic::{Action, ActionVec};

        let actions = vec![
            Action::new_builder()
                .script_info_hash(Self::rng_byte32())
                .data(Self::rng_bytes(30))
                .build(),
            Action::new_builder()
                .script_info_hash(Self::rng_byte32())
                .build(),
        ];

        let msg = Message::new_builder().actions(ActionVec::new_builder().set(actions).build());

        self.message_data.get_mut(index).unwrap().action = Some(msg.build());
    }

    pub fn get_witnesses(&self) -> Vec<Bytes> {
        let mut witnesses = Vec::new();

        for data in &self.message_data {
            let d = data.new_extended_witness();
            witnesses.push(d.as_bytes());

            for _ in 1..data.group_size {
                witnesses.push(Bytes::new());
            }
        }

        for w in &self.others {
            witnesses.push(w.as_bytes());
        }

        witnesses
    }

    pub fn get_action(&self) -> &Message {
        for d in &self.message_data {
            if d.action.is_some() {
                return d.action.as_ref().unwrap();
            }
        }
        panic!("none")
    }

    pub fn get_types_data_by_args(&self, args: &[u8]) -> &MessageData {
        assert_eq!(args.len(), 20);

        for d in &self.message_data {
            if d.pubkey_hash == args {
                return d;
            }
        }

        panic!("args cannot be found {:02x?}", args);
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
        context.deploy_cell(loader.load_binary("transaction-cobuild-lock-demo")),
        tx,
    )
}

pub fn gen_tx(witnesses: &MessageWitnesses) -> (TransactionView, Context) {
    let mut context = Context::default();
    let (out_point, mut tx) = append_cells(&mut context);

    for data in &witnesses.message_data {
        for _ in 0..data.group_size {
            let lock_script = context
                .build_script(&out_point, Bytes::from(data.pubkey_hash.to_vec()))
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

    for witness in tx.witnesses().into_iter().skip(tx.inputs().len()) {
        hasher.update(&(witness.len() as u32).to_le_bytes());
        hasher.update(&witness.raw_data());
    }

    let mut ret_hash = [0u8; 32];
    hasher.finalize(&mut ret_hash);
    ret_hash
}

fn witness_is_empty(tx: &TransactionView, index: usize) -> bool {
    let w = tx.witnesses().get(index);
    if w.is_none() {
        return true;
    }

    let w = w.unwrap();
    if w.is_empty() || w.len() == 4 {
        return true;
    }

    false
}

pub fn sign_tx(witnesses: &mut MessageWitnesses, tx: TransactionView) -> TransactionView {
    // get sign message
    let skeleton_hash = generate_skeleton_hash(&tx);

    let msg = witnesses.get_action().as_slice().to_vec();
    let mut data_count = 0usize;
    for i in 0..tx.inputs().len() {
        if witness_is_empty(&tx, i) {
            continue;
        }

        let mut hasher = new_blake2b();
        hasher.update(&(msg.len() as u32).to_le_bytes());
        hasher.update(&msg);
        hasher.update(&skeleton_hash);

        let mut message_digest = [0u8; 32];
        hasher.finalize(&mut message_digest);

        let sign = witnesses
            .message_data
            .get(data_count)
            .unwrap()
            .privkey
            .sign_recoverable(&SecpMessage::from_slice(&message_digest).unwrap())
            .expect("sign")
            .serialize();

        witnesses.message_data.get_mut(data_count).unwrap().sign = Some(sign);
        data_count += 1;
    }

    let witnesses = witnesses.get_witnesses();
    tx.as_advanced_builder()
        .set_witnesses(witnesses.iter().map(|f| f.pack()).collect())
        .build()
}
