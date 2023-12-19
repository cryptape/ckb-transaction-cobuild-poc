import { Address, Hash, HexString, Script, utils, blockchain } from '@ckb-lumos/base';
import { UnpackResult, bytes } from "@ckb-lumos/codec";
import { secp256k1Blake160 } from '@ckb-lumos/common-scripts';
import { RPC, hd, helpers } from '@ckb-lumos/lumos';
import { defaultEmptyWitnessArgs, isScriptValueEquals, updateWitnessArgs } from '@spore-sdk/core';
import { readFileSync } from 'fs';
import { List } from "immutable";
import { resolve } from 'path';
import { blockchainTransactionToAPITransaction, generateFinalHash, generateSkeletonHash } from './tmBuild';
import { config, configTransactionCobuildLockDemo } from './tmConfig';
import { ScriptInfo, SighashAll, Action, BuildingPacket, SporeAction, Message } from './tmMolecule';
import { txDump } from './txDump';
import 'process';
const { computeScriptHash, ckbHash } = utils;

export async function fetchLocalFile(src: string) {
  const buffer = readFileSync(resolve(__dirname, src));
  return new Uint8Array(buffer).buffer;
}

async function readline(): Promise<string> {
  return new Promise((resolve, reject) => {
    process.stdin.resume();
    process.stdin.on('data', function (data) {
      process.stdin.pause();
      resolve(data.toString());
    })
  })
}

export interface Wallet {
  lock: Script;
  address: Address;
  signMessage(message: HexString): Hash;
  signTransaction(txSkeleton: helpers.TransactionSkeletonType): helpers.TransactionSkeletonType;
  signAndSendTransaction(txSkeleton: helpers.TransactionSkeletonType): Promise<Hash>;
  signAndSendBuildingPacket(buildingPacket: Uint8Array): Promise<Hash>;
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
    signAndSendBuildingPacket: null,
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
    codeHash: configTransactionCobuildLockDemo.script.codeHash,
    hashType: configTransactionCobuildLockDemo.script.hashType,
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

  async function signAndSendTransaction(txSkeleton: helpers.TransactionSkeletonType): Promise<Hash> {
    return null;
  }

  // Sign the transaction and send it via RPC
  async function signAndSendBuildingPacket(buildingPacket: Uint8Array): Promise<Hash> {

    let bp = BuildingPacket.unpack(buildingPacket);
    let tx = bp.value.payload;

    for (let action of bp.value.message.actions) {
      let r = bp.value.scriptInfos.filter((x) => x.scriptHash == action.scriptHash!)
      if (r.length != 1) {
        throw `cannot found script hash ${action.scriptHash} in building packet script infos.`
      }
      console.log(`Dapp name: ${Buffer.from(bytes.bytify(r[0].name).buffer).toString()}`)
      console.log(`Dapp url: ${Buffer.from(bytes.bytify(r[0].url).buffer).toString()}`)
      let sporeAction = SporeAction.unpack(action.data)
      console.log(`Dapp action: ${JSON.stringify(sporeAction, null, 4)}`)
    }

    // console.log('Sign and send the message? [Y]es, [N]o')
    // let userInput = (await readline()).toUpperCase()[0]
    // if (userInput == 'N') {
    //   throw `User refuses to sign transaction`
    // }

    let skeletonHash = generateSkeletonHash(tx)
    let messageBytes = bytes.hexify(Message.pack(bp.value.message))
    let messageDigest = generateFinalHash(messageBytes, skeletonHash)
    let seal = signMessage(messageDigest)
    let sighashAll = SighashAll.pack({
      seal: seal,
      message: bp.value.message,
    })
    let witness0 = '0x010000ff' + bytes.hexify(sighashAll).slice(2);
    tx.witnesses[0] = witness0

    console.log(JSON.stringify(blockchainTransactionToAPITransaction(tx), null, 4))

    const rpc = new RPC(config.ckbNodeUrl);
    return await rpc.sendTransaction(blockchainTransactionToAPITransaction(tx), 'passthrough');
  }

  return {
    lock,
    address,
    signMessage,
    signTransaction,
    signAndSendTransaction,
    signAndSendBuildingPacket,
  };
}

export const tmAccounts = {
  alice: createTmLockWallet('0x0000000000000000000000000000000000000000000000000000000000000001'),
  bob: createTmLockWallet('0x0000000000000000000000000000000000000000000000000000000000000002'),
};
