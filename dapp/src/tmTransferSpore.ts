import { transferSpore } from '@spore-sdk/core';
import { config } from './tmConfig';
import { accounts } from './tmWallet';

async function main() {
    let { txSkeleton } = await transferSpore({
        outPoint: {
            txHash: '0xb83fa0529c76fede0531b211ddf61a689f52470584d9f487cd6c40a7df7cec53',
            index: '0x0',
        },
        toLock: accounts.bob.lock,
        config: config,
    });
    let hash = await accounts.alice.signAndSendTransaction(txSkeleton);
    console.log(`Spore transfered at: https://pudge.explorer.nervos.org/transaction/${hash}`);
}

main()
