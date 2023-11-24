import { Address, Hash, HexString, Script, blockchain, utils, values } from '@ckb-lumos/base';
import { BI } from "@ckb-lumos/bi";
import { bytes } from "@ckb-lumos/codec";
import { secp256k1Blake160 } from '@ckb-lumos/common-scripts';
import { Config } from "@ckb-lumos/config-manager";
import { TransactionSkeletonType, createTransactionFromSkeleton } from "@ckb-lumos/helpers";
import { RPC, hd, helpers } from '@ckb-lumos/lumos';
import { defaultEmptyWitnessArgs, isScriptValueEquals, updateWitnessArgs } from '@spore-sdk/core';
import { Set } from "immutable";
import { config, configTypedMessageLockDemo } from './tmConfig';

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
    txSkeleton = prepareSigningEntries(txSkeleton, config.lumos, 'SECP256K1_BLAKE160');
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

export const tmAccounts = {
  alice: createTmLockWallet('0x0000000000000000000000000000000000000000000000000000000000000001'),
  bob: createTmLockWallet('0x0000000000000000000000000000000000000000000000000000000000000002'),
};
