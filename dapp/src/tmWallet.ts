import { Address, Hash, HexString, Script, utils } from '@ckb-lumos/base';
import { UnpackResult, bytes } from "@ckb-lumos/codec";
import { secp256k1Blake160 } from '@ckb-lumos/common-scripts';
import { RPC, hd, helpers } from '@ckb-lumos/lumos';
import { defaultEmptyWitnessArgs, isScriptValueEquals, updateWitnessArgs } from '@spore-sdk/core';
import { readFileSync } from 'fs';
import { List } from "immutable";
import { resolve } from 'path';
import { generateFinalHash, generateSkeletonHash } from './tmBuild';
import { config, configTypedMessageLockDemo } from './tmConfig';
import { DappInfo, SighashWithAction, SigningAction, SporeAction, TypedMessage } from './tmMolecule';
const { computeScriptHash, ckbHash } = utils;

export async function fetchLocalFile(src: string) {
  const buffer = readFileSync(resolve(__dirname, src));
  return new Uint8Array(buffer).buffer;
}

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

    let signingAction = SigningAction.unpack(bytes.bytify(txSkeleton.signingEntries.get(0).message));
    console.log('signingAction', JSON.stringify(signingAction, null, 4))

    let action = SporeAction.unpack(signingAction.message.value[0].action.data);
    console.log('action', JSON.stringify(action, null, 4))

    let dappInfo: UnpackResult<typeof DappInfo> = {
      type: 'DappInfoV1',
      value: {
        name: bytes.hexify(bytes.bytifyRawString('spore')),
        url: bytes.hexify(bytes.bytifyRawString('https://a-simple-demo.spore.pro')),
        scriptHash: computeScriptHash({
          codeHash: config.scripts.Spore.script.codeHash,
          hashType: config.scripts.Spore.script.hashType,
          args: txSkeleton.outputs.get(0).cellOutput.type!.args,
        }),
        schema: bytes.hexify(new Uint8Array(await fetchLocalFile('../../schemas/spore.mol'))),
        messageType: bytes.hexify(bytes.bytifyRawString('SporeAction')),
      }
    };
    let dappInfoHash = ckbHash(DappInfo.pack(dappInfo));
    if (dappInfoHash != signingAction.message.value[0].action.infoHash) {
      throw 'Check infoHash failed!'
    }

    let skeletonHash = signingAction.skeletonHash
    let typedMessage = bytes.hexify(TypedMessage.pack(signingAction.message))
    let digestMessage = generateFinalHash(skeletonHash, typedMessage)
    let sighashWithActionLock = signMessage(digestMessage)
    let sighashWithAction = SighashWithAction.pack({
      lock: sighashWithActionLock,
      message: signingAction.message,
    })
    let witness0 = '0x010000ff' + bytes.hexify(sighashWithAction).slice(2);
    let witnesses = List<string>()
    witnesses = witnesses.set(0, witness0)
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
