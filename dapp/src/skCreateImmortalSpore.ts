import { createSpore } from '@spore-sdk/core';
import { readFileSync } from 'fs';
import { resolve } from 'path';
import { accounts } from './skWallet';

export async function fetchLocalFile(src: string) {
    const buffer = readFileSync(resolve(__dirname, src));
    return new Uint8Array(buffer).buffer;
}

export async function main() {
    let { txSkeleton, outputIndex } = await createSpore({
        data: {
            contentType: 'image/jpeg',
            content: await fetchLocalFile('../res/nervos.jpg'),
            contentTypeParameters: {
                immortal: true,
            },
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
