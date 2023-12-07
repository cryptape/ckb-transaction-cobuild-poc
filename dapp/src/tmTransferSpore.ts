import { blockchain, utils } from '@ckb-lumos/base';
import { UnpackResult, bytes } from "@ckb-lumos/codec";
import { common } from '@ckb-lumos/common-scripts';
import { BI } from '@ckb-lumos/lumos';
import { transferSpore } from '@spore-sdk/core';
import { createTransactionFromSkeleton } from "@ckb-lumos/helpers";
import { readFileSync } from 'fs';
import { resolve } from 'path';
import { generateSkeletonHash, setupInputCell } from './tmBuild';
import { config, configTransactionCobuildLockDemo } from './tmConfig';
import { Action, ActionVec, ScriptInfo, Message, SighashAll, SporeAction, BuildingPacket } from './tmMolecule';
import { tmAccounts } from './tmWallet';
const { ckbHash, computeScriptHash } = utils;
const { registerCustomLockScriptInfos } = common;

export async function fetchLocalFile(src: string) {
    const buffer = readFileSync(resolve(__dirname, src));
    return new Uint8Array(buffer).buffer;
}

async function main() {
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
        outPoint: {
            txHash: '0xab2a0f4073278bfdbf5e8b2ab2f7a984f5360452c7ec81b7c28ecc5758f43ec4',
            index: '0x0',
        },
        toLock: tmAccounts.bob.lock,
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
        schema: bytes.hexify(new Uint8Array(await fetchLocalFile('../../schemas/spore.mol'))),
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


    // let signingEntries = txSkeleton.get("signingEntries");
    // signingEntries = signingEntries.push({
    //     type: 'typedMessage',
    //     index: 0,
    //     message: bytes.hexify(SigningAction.pack(signingAction)),
    // });
    // txSkeleton = txSkeleton.set('signingEntries', signingEntries)

    let hash = await tmAccounts.alice.signAndSendBuildingPacket(buildingPacket);

    console.log(`Spore transfered at: https://pudge.explorer.nervos.org/transaction/${hash}`);
}

main()
