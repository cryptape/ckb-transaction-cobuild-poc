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
            txHash: '0x80db6310bd410070e7003302a0e13bb8c441a965c974bdefb16d3afa69bfccea',
            index: '0x0',
        },
        toLock: tmAccounts.bob.lock,
        config: config,
    });
    let hash = await tmAccounts.alice.signAndSendTransaction(txSkeleton);
    console.log(`Spore transfered at: https://pudge.explorer.nervos.org/transaction/${hash}`);
}

main()
