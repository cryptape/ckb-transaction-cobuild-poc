import { bytes } from "@ckb-lumos/codec";
import { Indexer, helpers, Address, Script, RPC, hd, config as lumosConfig, Cell, commons, WitnessArgs, BI, HexString, OutPoint } from "@ckb-lumos/lumos";
import { values, blockchain } from "@ckb-lumos/base";
import { option } from "@ckb-lumos/codec/lib/molecule";
import { config } from "./tmConfig"
import { tmAccounts, Wallet } from "./tmWallet"
import { unpackToRawSporeData, RawSporeData } from '@spore-sdk/core';

// There is a bug in Lomus: https://github.com/ckb-js/lumos/pull/583
// Dirty fix it without update lumos.
require('abort-controller').AbortController = require('abort-controller').default

interface SporeContainer {
  outPoint: OutPoint,
  sporeID: HexString,
  spore: RawSporeData,
  b64Data: string,
}

export async function querySporeCells(lock: Script): Promise<Array<SporeContainer>> {
  const rpc = new RPC(config.ckbNodeUrl);
  let cells = await rpc.getCells({
    script: lock,
    scriptType: 'lock',
    filter: {
      script: {
        codeHash: config.scripts.Spore.script.codeHash,
        hashType: config.scripts.Spore.script.hashType,
        args: "0x"
      }
    }
  }, 'asc', '0x100', null); // Only query max 256 spores in this demo.
  let myout = []
  for (let e of cells.objects) {
    myout.push({
      outPoint: e.outPoint,
      sporeID: e.output.type!.args,
      spore: unpackToRawSporeData(e.outputData),
      b64Data: "data:image/png;base64," + Buffer.from(unpackToRawSporeData(e.outputData).content.slice(2), 'hex').toString('base64')
    })
  }
  return myout
}

import { utils } from '@ckb-lumos/base';
import { UnpackResult } from "@ckb-lumos/codec";
import { common } from '@ckb-lumos/common-scripts';
import { transferSpore } from '@spore-sdk/core';
import { createTransactionFromSkeleton } from "@ckb-lumos/helpers";
import { readFileSync } from 'fs';
import { resolve } from 'path';
import { generateSkeletonHash, setupInputCell } from './tmBuild';
import { configTransactionCobuildLockDemo } from './tmConfig';
import { Action, ActionVec, ScriptInfo, Message, SighashAll, SporeAction, BuildingPacket } from './tmMolecule';
const { ckbHash, computeScriptHash } = utils;
const { registerCustomLockScriptInfos } = common;
import { blockchainTransactionToAPITransaction, generateFinalHash } from './tmBuild';

const schema = `import blockchain;
import basic;

union Address {
    Script,
}

option AddressOpt (Address);

table Mint {
    id: Byte32,
    to: Address,
    content_hash: Byte32,
}

table Transfer {
    nft_id: Byte32,
    from: AddressOpt,
    to: AddressOpt,
}

table Melt {
    id: Byte32,
}

union SporeAction {
    Mint,
    Transfer,
    Melt,
}
`

export async function createBuildingPacket(to: Script, outPoint: OutPoint) {
    registerCustomLockScriptInfos([
        {
            codeHash: configTransactionCobuildLockDemo.script.codeHash,
            hashType: configTransactionCobuildLockDemo.script.hashType,
            lockScriptInfo: {
                CellCollector: null,
                setupInputCell: setupInputCell,
                prepareSigningEntries: null,
                setupOutputCell: null,
            },
        },
    ])

    let { txSkeleton } = await transferSpore({
        outPoint: outPoint,
        toLock: to,
        config: config,
    });

    let scriptInfo: UnpackResult<typeof ScriptInfo> = {
        name: bytes.hexify(bytes.bytifyRawString('spore')),
        url: bytes.hexify(bytes.bytifyRawString('https://a-simple-demo.spore.pro')),
        scriptHash: computeScriptHash({
            codeHash: config.scripts.Spore.script.codeHash,
            hashType: config.scripts.Spore.script.hashType,
            args: txSkeleton.outputs.get(0).cellOutput.type!.args,
        }),
        schema: bytes.hexify(bytes.bytifyRawString(schema)),
        messageType: bytes.hexify(bytes.bytifyRawString('SporeAction')),
    };
    let scriptInfoHash = ckbHash(ScriptInfo.pack(scriptInfo));
    let sporeID = txSkeleton.outputs.get(0).cellOutput.type!.args;
    let sporeTransferTo = txSkeleton.outputs.get(0).cellOutput.lock;
    let actionData = bytes.hexify(SporeAction.pack({
        type: 'Transfer',
        value: {
            nftID: sporeID,
            to: {
                type: 'Script',
                value: sporeTransferTo
            },
        },
    }))
    let action: UnpackResult<typeof Action> = {
        scriptInfoHash: scriptInfoHash,
        scriptHash: scriptInfo.scriptHash,
        data: actionData,
    };
    let message: UnpackResult<typeof Message> = {
        actions: [action],
    };
    let sighashAll = SighashAll.pack({
        seal: '0x' + '0'.repeat(130),
        message: message,
    })
    let witness0 = '0x010000ff' + bytes.hexify(sighashAll).slice(2);
    let extraFee = (witness0.length - 2) / 2 - 85
    txSkeleton.outputs.get(0).cellOutput.capacity = '0x' + (parseInt(txSkeleton.outputs.get(0).cellOutput.capacity, 16) - extraFee).toString(16)

    let buildingPacket = BuildingPacket.pack({
        type: 'BuildingPacketV1',
        value: {
            message: message,
            payload: createTransactionFromSkeleton(txSkeleton),
            scriptInfos: [scriptInfo],
            lockActions: [],
        }
    })
    return buildingPacket
}

export async function giveMessage(buildingPacket: Uint8Array) {
  let bp = BuildingPacket.unpack(buildingPacket);
  let tx = bp.value.payload;

  let alertMessage = ""

  for (let action of bp.value.message.actions) {
    let r = bp.value.scriptInfos.filter((x) => x.scriptHash == action.scriptHash!)
    if (r.length != 1) {
      throw `cannot found script hash ${action.scriptHash} in building packet script infos.`
    }
    alertMessage += `Dapp name: ${Buffer.from(bytes.bytify(r[0].name).buffer).toString()}`
    alertMessage += "\n"
    alertMessage += `Dapp url: ${Buffer.from(bytes.bytify(r[0].url).buffer).toString()}`
    alertMessage += "\n"
    let sporeAction = SporeAction.unpack(action.data)
    alertMessage += `Dapp action: ${JSON.stringify(sporeAction, null, 4)}`
    alertMessage += "\n"
  }
  alertMessage += 'Sign and send the message? [Y]es, [N]o'
  alertMessage += "\n"
  return alertMessage
}

export async function transfer(from: Wallet, to: Script, outPoint: OutPoint) {
  let buildingPacket = await createBuildingPacket(to, outPoint);
  let bp = BuildingPacket.unpack(buildingPacket);
  let tx = bp.value.payload;

  let alertMessage = ""

  for (let action of bp.value.message.actions) {
    let r = bp.value.scriptInfos.filter((x) => x.scriptHash == action.scriptHash!)
    if (r.length != 1) {
      throw `cannot found script hash ${action.scriptHash} in building packet script infos.`
    }
    alertMessage += `Dapp name: ${Buffer.from(bytes.bytify(r[0].name).buffer).toString()}`
    alertMessage += "\n"
    alertMessage += `Dapp url: ${Buffer.from(bytes.bytify(r[0].url).buffer).toString()}`
    alertMessage += "\n"
    let sporeAction = SporeAction.unpack(action.data)
    alertMessage += `Dapp action: ${JSON.stringify(sporeAction, null, 4)}`
    alertMessage += "\n"
  }
  alertMessage += 'Sign and send the message? [Y]es, [N]o'
  alertMessage += "\n"
  alert(alertMessage)

  let skeletonHash = generateSkeletonHash(tx)
  let messageBytes = bytes.hexify(Message.pack(bp.value.message))
  let messageDigest = generateFinalHash(skeletonHash, messageBytes)
  let seal = from.signMessage(messageDigest)
  let sighashAll = SighashAll.pack({
    seal: seal,
    message: bp.value.message,
  })
  let witness0 = '0x010000ff' + bytes.hexify(sighashAll).slice(2);
  tx.witnesses[0] = witness0

  const rpc = new RPC(config.ckbNodeUrl);
  return await rpc.sendTransaction(blockchainTransactionToAPITransaction(tx), 'passthrough');
}
