import { Script, Indexer, BI, HexString, OutPoint } from "@ckb-lumos/lumos";
import { config } from './tmConfig';
import { tmAccounts } from "./tmWallet";
import { accounts as skAccounts } from "./skWallet";
const indexer = new Indexer(config.ckbIndexerUrl);
const spore = config.scripts.Spore.script;

async function getSporeCells(lock: Script): Promise<Array<{args: HexString, outPoint: OutPoint}>> {
    const spore_type_script_tmpl = {
        codeHash: spore.codeHash,
        hashType: spore.hashType,
        args: "0x"
    };
    const collector = indexer.collector({ lock, type: {script: spore_type_script_tmpl, searchMode: "prefix", argsLen: 0} });
    let result = [];
    for await (const cell of collector.collect()) {
        let type = cell.cellOutput.type!;
        result.push({args: type.args, outPoint: cell.outPoint});
    }
    return result;
}

// it costs a lot of time if the own has lots of UTXO
async function getBalance(lock: Script): Promise<BI> {
    const collector = indexer.collector({ lock });
    let balance: BI = BI.from(0);
    let count = 0;
    for await (const cell of collector.collect()) {
        balance = balance.add(cell.cellOutput.capacity)
        count += 1;
        if (count > 256) {
            console.log("too much UTXO(> 256), abort")
            break
        }
    }
    return balance.div(BI.from(10).pow(8));
}

async function main() {
    console.log("Alice's spore cells(display outPoint and args):")
    let alice_cells = await getSporeCells(tmAccounts.alice.lock)
    for (let cell of alice_cells) {
        console.log("outPoint =", cell.outPoint, "args = ", cell.args);
    }
    console.log("Bob's spore cells(display outPoint and args):")
    let bob_cells = await getSporeCells(tmAccounts.bob.lock);
    for (let cell of bob_cells) {
        console.log("outPoint = ", cell.outPoint, "args =", cell.args);
    }
    let alice_balance = await getBalance(skAccounts.alice.lock);
    console.log(`alice's balance is ${alice_balance}`)
    let bob_balance = await getBalance(skAccounts.bob.lock);
    console.log(`bob's balance is ${bob_balance}`)
}

main();

