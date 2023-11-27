import { common } from '@ckb-lumos/common-scripts';
import { transferSpore } from '@spore-sdk/core';
import { setupInputCell } from './tmBuild';
import { config, configTypedMessageLockDemo } from './tmConfig';
import { tmAccounts } from './tmWallet';
const { registerCustomLockScriptInfos } = common;

async function main() {
    registerCustomLockScriptInfos([
        {
            codeHash: configTypedMessageLockDemo.script.codeHash,
            hashType: configTypedMessageLockDemo.script.hashType,
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
            txHash: '0x284bd6cc2633b3cfcab64b704b979f35b4145cc6b7f751e676be828d21d35807',
            index: '0x0',
        },
        toLock: tmAccounts.bob.lock,
        config: config,
    });
    let hash = await tmAccounts.alice.signAndSendTransaction(txSkeleton);
    console.log(`Spore transfered at: https://pudge.explorer.nervos.org/transaction/${hash}`);
}

main()
