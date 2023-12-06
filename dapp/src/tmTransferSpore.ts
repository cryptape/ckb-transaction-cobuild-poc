import { utils } from '@ckb-lumos/base';
import { UnpackResult, bytes } from "@ckb-lumos/codec";
import { common } from '@ckb-lumos/common-scripts';
import { BI } from '@ckb-lumos/lumos';
import { transferSpore } from '@spore-sdk/core';
import { readFileSync } from 'fs';
import { resolve } from 'path';
import { generateSkeletonHash, setupInputCell } from './tmBuild';
import { config, configTransactionCobuildLockDemo } from './tmConfig';
import { Action, DappInfo, ScriptAction, SighashWithAction, SigningAction, SporeAction } from './tmMolecule';
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
            txHash: '0x4e257666e1fb5ed06badcb9322b53c20bb90956660906b0f70462cae8857978a',
            index: '0x0',
        },
        toLock: tmAccounts.bob.lock,
        config: config,
    });

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
        infoHash: dappInfoHash,
        data: actionData,
    };
    let scriptAction: UnpackResult<typeof ScriptAction> = {
        scriptHash: dappInfo.value.scriptHash,
        action: action,
    };
    let signingActionMessage = [scriptAction];
    let signingActionSignature = '0x' + '0'.repeat(130);
    let sighashWithAction = SighashWithAction.pack({
        lock: signingActionSignature,
        message: {
            type: 'TypedMessageV1',
            value: signingActionMessage,
        },
    })
    let witness0 = '0x010000ff' + bytes.hexify(sighashWithAction).slice(2);
    let extraFee = (witness0.length - 2) / 2 - 85
    txSkeleton.outputs.get(0).cellOutput.capacity = '0x' + (parseInt(txSkeleton.outputs.get(0).cellOutput.capacity, 16) - extraFee).toString(16)
    let signingAction: UnpackResult<typeof SigningAction> = {
        flags: BI.from(0),
        address: {
            type: 'Script',
            value: txSkeleton.inputs.get(0).cellOutput.lock,
        },
        message: {
            type: 'TypedMessageV1',
            value: signingActionMessage,
        },
        skeletonHash: generateSkeletonHash(txSkeleton),
        infos: [dappInfo],
        scratch: null,
    }

    let signingEntries = txSkeleton.get("signingEntries");
    signingEntries = signingEntries.push({
        type: 'typedMessage',
        index: 0,
        message: bytes.hexify(SigningAction.pack(signingAction)),
    });
    txSkeleton = txSkeleton.set('signingEntries', signingEntries)
    let hash = await tmAccounts.alice.signAndSendTransaction(txSkeleton);
    console.log(`Spore transfered at: https://pudge.explorer.nervos.org/transaction/${hash}`);
}

main()
