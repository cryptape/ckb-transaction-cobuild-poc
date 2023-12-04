import { Script, Indexer, BI, HexString, OutPoint, RPC } from "@ckb-lumos/lumos";
import { config } from './tmConfig';
import { tmAccounts } from "./tmWallet";
import { accounts as skAccounts } from "./skWallet";


const indexer = new Indexer(config.ckbIndexerUrl);
const spore = config.scripts.Spore.script;

async function querySporeCells(lock: Script): Promise<Array<{ args: HexString, outPoint: OutPoint }>> {
    const spore_type_script_tmpl = {
        codeHash: spore.codeHash,
        hashType: spore.hashType,
        args: "0x"
    };
    const collector = indexer.collector({ lock, type: { script: spore_type_script_tmpl, searchMode: "prefix", argsLen: 0 } });
    let result = [];
    for await (const cell of collector.collect()) {
        let type = cell.cellOutput.type!;
        result.push({ args: type.args, outPoint: cell.outPoint });
    }
    return result;
}

// It costs a lot of time if a owner has lots of UTXO
async function queryBalance(lock: Script): Promise<BI> {
    const rpc = new RPC(config.ckbNodeUrl);
    let r = await rpc.getCellsCapacity({
        script: lock,
        scriptType: 'lock',
        filter: {
            scriptLenRange: ['0x0', '0x1']
        }
    })
    return BI.from(r.capacity).div(BI.from(10).pow(8))
}

async function main() {
    console.log("Alice's spore cells(display outPoint and args):")
    let alice_cells = await querySporeCells(tmAccounts.alice.lock)
    for (let cell of alice_cells) {
        console.log("outPoint =", cell.outPoint, "args = ", cell.args);
    }
    console.log("Bob's spore cells(display outPoint and args):")
    let bob_cells = await querySporeCells(tmAccounts.bob.lock);
    for (let cell of bob_cells) {
        console.log("outPoint = ", cell.outPoint, "args =", cell.args);
    }
    let alice_balance = await queryBalance(skAccounts.alice.lock);
    console.log(`alice's balance is ${alice_balance} CKByte`)
    let bob_balance = await queryBalance(skAccounts.bob.lock);
    console.log(`bob's balance is ${bob_balance} CKByte`)
}

main();
