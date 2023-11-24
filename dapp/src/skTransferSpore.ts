import { transferSpore } from '@spore-sdk/core';
import { accounts } from './skWallet';

async function main() {
    let { txSkeleton } = await transferSpore({
        outPoint: {
            txHash: '0x4039a99666eb2c95d7d78c7d0026246d1739758d1308e979a30c2d33eed5d40e',
            index: '0x0',
        },
        toLock: accounts.bob.lock,
    });
    let hash = await accounts.alice.signAndSendTransaction(txSkeleton);
    console.log(`Spore transfered at: https://pudge.explorer.nervos.org/transaction/${hash}`);
}

main()
