import { createCluster } from '@spore-sdk/core';
import { accounts } from './skWallet';

export async function main() {
    let { txSkeleton, outputIndex } = await createCluster({
        data: {
            name: 'ohayou',
            description: '',
        },
        toLock: accounts.alice.lock,
        fromInfos: [accounts.alice.address],
    });
    const hash = await accounts.alice.signAndSendTransaction(txSkeleton)
    console.log(`Cluster created at: https://pudge.explorer.nervos.org/transaction/${hash}`);
    let clusterID = txSkeleton.get('outputs').get(outputIndex)!.cellOutput.type!.args;
    // Cluster ID: 0x66d97508a7e81acd10e6845eec90d564df35dc0bfd2338796b9677f3738a3614
    console.log(`Cluster ID: ${clusterID}`);
    console.log(`Cluster: https://a-simple-demo.spore.pro/cluster/${clusterID}`);
}

main()
