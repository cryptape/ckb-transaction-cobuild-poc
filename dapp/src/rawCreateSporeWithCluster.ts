import { createSpore } from '@spore-sdk/core';
import { readFileSync } from 'fs';
import { resolve } from 'path';
import { accounts } from './rawWallet';

export async function fetchLocalFile(src: string) {
    const buffer = readFileSync(resolve(__dirname, src));
    return new Uint8Array(buffer).buffer;
}

export async function main() {
    const { txSkeleton, outputIndex } = await createSpore({
        data: {
            contentType: 'image/jpeg',
            content: await fetchLocalFile('../res/nervos.jpg'),
            clusterId: '0x66d97508a7e81acd10e6845eec90d564df35dc0bfd2338796b9677f3738a3614',
        },
        toLock: accounts.alice.lock,
        fromInfos: [accounts.alice.address],
    });
    const hash = await accounts.alice.signAndSendTransaction(txSkeleton);
    console.log(`Spore created at: https://pudge.explorer.nervos.org/transaction/${hash}`);
    let sporeID = txSkeleton.get('outputs').get(outputIndex)!.cellOutput.type!.args;
    console.log(`Spore ID: ${sporeID}`);
    console.log(`Spore: https://a-simple-demo.spore.pro/spore/${sporeID}`);
}

main()
