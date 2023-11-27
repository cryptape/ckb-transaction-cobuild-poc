import { Address, Hash, HexString, Script, blockchain, utils, values } from '@ckb-lumos/base';
import { BI } from "@ckb-lumos/bi";
import { UnpackResult, bytes, number } from "@ckb-lumos/codec";
import { secp256k1Blake160 } from '@ckb-lumos/common-scripts';
import { Config } from "@ckb-lumos/config-manager";
import { TransactionSkeletonType, createTransactionFromSkeleton } from "@ckb-lumos/helpers";
import { RPC, hd, helpers } from '@ckb-lumos/lumos';
import { defaultEmptyWitnessArgs, isScriptValueEquals, updateWitnessArgs } from '@spore-sdk/core';
import { List, Set } from "immutable";
import { config, configTypedMessageLockDemo } from './tmConfig';
import { SighashWithAction, TypedMessage } from './tmMolecule';
const { Uint64, Uint32 } = number;

const { CKBHasher, ckbHash } = utils;

export interface Wallet {
  lock: Script;
  address: Address;
  signMessage(message: HexString): Hash;
  signTransaction(txSkeleton: helpers.TransactionSkeletonType): helpers.TransactionSkeletonType;
  signAndSendTransaction(txSkeleton: helpers.TransactionSkeletonType): Promise<Hash>;
}

/**
 * Create a CKB Default Lock (Secp256k1Blake160 Sign-all) Wallet by a private-key and a SporeConfig,
 * providing lock/address, and functions to sign message/transaction and send the transaction on-chain.
 */
export function createSkLockWallet(privateKey: HexString): Wallet {
  // Generate a lock script from the private key
  const Secp256k1Blake160 = config.lumos.SCRIPTS['SECP256K1_BLAKE160']!;
  const lock: Script = {
    codeHash: Secp256k1Blake160.CODE_HASH,
    hashType: Secp256k1Blake160.HASH_TYPE,
    args: hd.key.privateKeyToBlake160(privateKey),
  };

  // Generate address from the lock script
  const address = helpers.encodeToAddress(lock, {
    config: config.lumos,
  });

  // Sign for a message
  function signMessage(message: HexString): Hash {
    return hd.key.signRecoverable(message, privateKey);
  }

  // Sign prepared signing entries,
  // and then fill signatures into Transaction.witnesses
  function signTransaction(txSkeleton: helpers.TransactionSkeletonType): helpers.TransactionSkeletonType {
    const signingEntries = txSkeleton.get('signingEntries');
    const signatures = new Map<HexString, Hash>();
    const inputs = txSkeleton.get('inputs');

    let witnesses = txSkeleton.get('witnesses');
    for (let i = 0; i < signingEntries.size; i++) {
      const entry = signingEntries.get(i)!;
      if (entry.type === 'witness_args_lock') {
        // Skip if the input's lock does not match to the wallet's lock
        const input = inputs.get(entry.index);
        if (!input || !isScriptValueEquals(input.cellOutput.lock, lock)) {
          continue;
        }

        // Sign message
        if (!signatures.has(entry.message)) {
          const sig = signMessage(entry.message);
          signatures.set(entry.message, sig);
        }

        // Update signature to Transaction.witnesses
        const signature = signatures.get(entry.message)!;
        const witness = witnesses.get(entry.index, defaultEmptyWitnessArgs);
        witnesses = witnesses.set(entry.index, updateWitnessArgs(witness, 'lock', signature));
      }
    }

    return txSkeleton.set('witnesses', witnesses);
  }

  // Sign the transaction and send it via RPC
  async function signAndSendTransaction(txSkeleton: helpers.TransactionSkeletonType): Promise<Hash> {
    // 1. Sign transaction
    txSkeleton = secp256k1Blake160.prepareSigningEntries(txSkeleton, { config: config.lumos });
    txSkeleton = signTransaction(txSkeleton);

    // 2. Convert TransactionSkeleton to Transaction
    const tx = helpers.createTransactionFromSkeleton(txSkeleton);

    // 3. Send transaction
    const rpc = new RPC(config.ckbNodeUrl);
    return await rpc.sendTransaction(tx, 'passthrough');
  }

  return {
    lock,
    address,
    signMessage,
    signTransaction,
    signAndSendTransaction,
  };
}

export const skAccounts = {
  alice: createSkLockWallet('0x0000000000000000000000000000000000000000000000000000000000000001'),
  bob: createSkLockWallet('0x0000000000000000000000000000000000000000000000000000000000000002'),
};


export function hashWitness(
  hasher: { update: (value: HexString | ArrayBuffer) => unknown },
  witness: HexString
): void {
  const lengthBuffer = new ArrayBuffer(8);
  const view = new DataView(lengthBuffer);
  const witnessHexString = BI.from(bytes.bytify(witness).length).toString(16);
  if (witnessHexString.length <= 8) {
    view.setUint32(0, Number("0x" + witnessHexString), true);
    view.setUint32(4, Number("0x" + "00000000"), true);
  }

  if (witnessHexString.length > 8 && witnessHexString.length <= 16) {
    view.setUint32(0, Number("0x" + witnessHexString.slice(-8)), true);
    view.setUint32(4, Number("0x" + witnessHexString.slice(0, -8)), true);
  }
  hasher.update(lengthBuffer);
  hasher.update(witness);
}

export function prepareSigningEntries(
  txSkeleton: TransactionSkeletonType,
  config: Config,
  scriptType: "SECP256K1_BLAKE160" | "SECP256K1_BLAKE160_MULTISIG" | "OMNILOCK"
): TransactionSkeletonType {
  const template = config.SCRIPTS[scriptType];
  if (!template) {
    throw new Error(
      `Provided config does not have ${scriptType} script setup!`
    );
  }
  let processedArgs = Set<string>();
  const tx = createTransactionFromSkeleton(txSkeleton);
  const txHash = ckbHash(blockchain.RawTransaction.pack(tx));
  const inputs = txSkeleton.get("inputs");
  const witnesses = txSkeleton.get("witnesses");
  let signingEntries = txSkeleton.get("signingEntries");
  for (let i = 0; i < inputs.size; i++) {
    const input = inputs.get(i)!;
    if (
      template.CODE_HASH === input.cellOutput.lock.codeHash &&
      template.HASH_TYPE === input.cellOutput.lock.hashType &&
      !processedArgs.has(input.cellOutput.lock.args)
    ) {
      processedArgs = processedArgs.add(input.cellOutput.lock.args);
      const lockValue = new values.ScriptValue(input.cellOutput.lock, {
        validate: false,
      });
      const hasher = new CKBHasher();
      hasher.update(txHash);
      if (i >= witnesses.size) {
        throw new Error(
          `The first witness in the script group starting at input index ${i} does not exist, maybe some other part has invalidly tampered the transaction?`
        );
      }
      hashWitness(hasher, witnesses.get(i)!);
      for (let j = i + 1; j < inputs.size && j < witnesses.size; j++) {
        const otherInput = inputs.get(j)!;
        if (
          lockValue.equals(
            new values.ScriptValue(otherInput.cellOutput.lock, {
              validate: false,
            })
          )
        ) {
          hashWitness(hasher, witnesses.get(j)!);
        }
      }
      for (let j = inputs.size; j < witnesses.size; j++) {
        hashWitness(hasher, witnesses.get(j)!);
      }
      const signingEntry = {
        type: "witness_args_lock",
        index: i,
        message: hasher.digestHex(),
      };
      signingEntries = signingEntries.push(signingEntry);
    }
  }
  txSkeleton = txSkeleton.set("signingEntries", signingEntries);
  return txSkeleton;
}

export function generateSkeletonHash(txSkeleton: TransactionSkeletonType): HexString {
  let data = ''

  const tx = createTransactionFromSkeleton(txSkeleton);
  const txHash = ckbHash(blockchain.RawTransaction.pack(tx));
  console.log('txHash', txHash)
  data += txHash

  for (let i = txSkeleton.inputs.size; i < txSkeleton.witnesses.size; i++) {
    const witness = txSkeleton.witnesses.get(i)
    console.log('hashWitness', witness)
    data += bytes.hexify(Uint32.pack(witness.length / 2 - 1)).slice(2)
    data += witness.slice(2)
  }
  return ckbHash(data)
}

export function generateFinalHash(skeletonHash: HexString, typedMessage: HexString): HexString {
  let data = ''
  data += skeletonHash
  data += bytes.hexify(Uint64.pack(typedMessage.length / 2 - 1)).slice(2)
  data += typedMessage.slice(2)
  return ckbHash(data)
}


/**
 * Create a CKB Default Lock (Secp256k1Blake160 Sign-all) Wallet by a private-key and a SporeConfig,
 * providing lock/address, and functions to sign message/transaction and send the transaction on-chain.
 */
export function createTmLockWallet(privateKey: HexString): Wallet {
  // Generate a lock script from the private key
  const lock: Script = {
    codeHash: configTypedMessageLockDemo.script.codeHash,
    hashType: configTypedMessageLockDemo.script.hashType,
    args: hd.key.privateKeyToBlake160(privateKey),
  };

  // Generate address from the lock script
  const address = helpers.encodeToAddress(lock, {
    config: config.lumos,
  });

  // Sign for a message
  function signMessage(message: HexString): Hash {
    return hd.key.signRecoverable(message, privateKey);
  }

  // Sign prepared signing entries,
  // and then fill signatures into Transaction.witnesses
  function signTransaction(txSkeleton: helpers.TransactionSkeletonType): helpers.TransactionSkeletonType {
    const signingEntries = txSkeleton.get('signingEntries');
    const signatures = new Map<HexString, Hash>();
    const inputs = txSkeleton.get('inputs');

    let witnesses = txSkeleton.get('witnesses');
    for (let i = 0; i < signingEntries.size; i++) {
      const entry = signingEntries.get(i)!;
      if (entry.type === 'witness_args_lock') {
        // Skip if the input's lock does not match to the wallet's lock
        const input = inputs.get(entry.index);
        if (!input || !isScriptValueEquals(input.cellOutput.lock, lock)) {
          continue;
        }

        // Sign message
        if (!signatures.has(entry.message)) {
          const sig = signMessage(entry.message);
          signatures.set(entry.message, sig);
        }

        // Update signature to Transaction.witnesses
        const signature = signatures.get(entry.message)!;
        const witness = witnesses.get(entry.index, defaultEmptyWitnessArgs);
        witnesses = witnesses.set(entry.index, updateWitnessArgs(witness, 'lock', signature));
      }
    }

    return txSkeleton.set('witnesses', witnesses);
  }

  // Sign the transaction and send it via RPC
  async function signAndSendTransaction(txSkeleton: helpers.TransactionSkeletonType): Promise<Hash> {
    // 1. Sign transaction
    // txSkeleton = prepareSigningEntries(txSkeleton, config.lumos, 'SECP256K1_BLAKE160');
    // txSkeleton = signTransaction(txSkeleton);

    let sighashWithActionLock = '0x39e370ad1e0ddf2717c78e8e4999f01d064936017ae83e26f6737702d29f468f75fec0aa2aa6fe78c27abc6db013acb3d0e5d1fbf911169faa3a4ac98f39e74800'
    let sighashWithActionMessage: UnpackResult<typeof TypedMessage> = {
      type: "TypedMessageV1",
      value: [{
        scriptHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
        action: {
          infoHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
          data: '0x1234567890'
        }
      }],
    };
    let sighashWithAction = SighashWithAction.pack({
      lock: sighashWithActionLock,
      message: sighashWithActionMessage,
    })
    let witnessIndex0 = '0x010000ff' + bytes.hexify(sighashWithAction).slice(2);
    let extraFee = (witnessIndex0.length - 2) / 2 - 85
    txSkeleton.outputs.get(0).cellOutput.capacity = '0x' + (parseInt(txSkeleton.outputs.get(0).cellOutput.capacity, 16) - extraFee).toString(16)

    let skeletonHash = generateSkeletonHash(txSkeleton)
    console.log('skeletonHash', skeletonHash)
    let typedMessage = bytes.hexify(TypedMessage.pack(sighashWithActionMessage))
    console.log('typedMessage', typedMessage)
    let digestMessage = generateFinalHash(skeletonHash, typedMessage)
    console.log('digestMessage', digestMessage)
    sighashWithActionLock = signMessage(digestMessage)
    console.log('signature', sighashWithActionLock)

    sighashWithAction = SighashWithAction.pack({
      lock: sighashWithActionLock,
      message: sighashWithActionMessage,
    })
    witnessIndex0 = '0x010000ff' + bytes.hexify(sighashWithAction).slice(2);

    let witnesses = List<string>()
    witnesses = witnesses.set(0, witnessIndex0)
    txSkeleton = txSkeleton.set('witnesses', witnesses)

    // 2. Convert TransactionSkeleton to Transaction
    const tx = helpers.createTransactionFromSkeleton(txSkeleton);
    console.log(JSON.stringify(tx, null, 4))

    // 3. Send transaction
    const rpc = new RPC(config.ckbNodeUrl);
    return await rpc.sendTransaction(tx, 'passthrough');
  }

  return {
    lock,
    address,
    signMessage,
    signTransaction,
    signAndSendTransaction,
  };
}

export const tmAccounts = {
  alice: createTmLockWallet('0x0000000000000000000000000000000000000000000000000000000000000001'),
  bob: createTmLockWallet('0x0000000000000000000000000000000000000000000000000000000000000002'),
};
