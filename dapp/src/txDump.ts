import { blockchain } from "@ckb-lumos/base";
import { molecule } from "@ckb-lumos/codec";
import { RPC, Transaction } from "@ckb-lumos/lumos";
import { ParamsFormatter } from "@ckb-lumos/rpc";
import { writeFile } from 'fs';
import { config } from './tmConfig';

// This function accepts a transaction object and converts this transaction
// object into a tx.json file supported by ckb-debugger.
export async function txDump(tx: Transaction, path: string) {
    const rpc = new RPC(config.ckbNodeUrl)

    let dumps = {}
    dumps['mock_info'] = {
        'inputs': [],
        'cell_deps': [],
        'header_deps': []
    }
    dumps['tx'] = ParamsFormatter.toRawTransaction(tx)

    for (const input of tx.inputs) {
        let currentTx = await rpc.getTransaction(input.previousOutput.txHash)
        let currentOutput = currentTx.transaction.outputs[parseInt(input.previousOutput.index, 16)]
        let currentData = currentTx.transaction.outputsData[parseInt(input.previousOutput.index, 16)]
        let currentHeader = currentTx.txStatus.blockHash;
        dumps['mock_info']['inputs'].push({
            input: ParamsFormatter.toInput(input),
            output: ParamsFormatter.toOutput(currentOutput),
            data: currentData,
            header: currentHeader
        });
    }

    for (const cellDep of tx.cellDeps) {
        let currentTx = await rpc.getTransaction(cellDep.outPoint.txHash)
        let currentOutput = currentTx.transaction.outputs[parseInt(cellDep.outPoint.index, 16)]
        let currentData = currentTx.transaction.outputsData[parseInt(cellDep.outPoint.index, 16)]
        let currentHeader = currentTx.txStatus.blockHash;
        dumps['mock_info']['cell_deps'].push({
            cell_dep: ParamsFormatter.toCellDep(cellDep),
            output: ParamsFormatter.toOutput(currentOutput),
            data: currentData,
            header: currentHeader,
        })
        if (cellDep.depType === "depGroup") {
            let cellDepGroup = molecule.vector(blockchain.OutPoint).unpack(currentData);
            for (const cellDepItem of cellDepGroup) {
                let currentTx = await rpc.getTransaction(cellDepItem.txHash)
                let currentOutput = currentTx.transaction.outputs[cellDepItem.index]
                let currentData = currentTx.transaction.outputsData[cellDepItem.index]
                let currentHeader = currentTx.txStatus.blockHash;
                dumps['mock_info']['cell_deps'].push({
                    cell_dep: ParamsFormatter.toCellDep({
                        outPoint: {
                            txHash: cellDepItem.txHash,
                            index: '0x' + cellDepItem.index.toString(16)
                        },
                        depType: "code",
                    }),
                    output: ParamsFormatter.toOutput(currentOutput),
                    data: currentData,
                    header: currentHeader,
                })
            }
        }
    }

    for (const headerDep of tx.headerDeps) {
        let header = await rpc.getHeader(headerDep)
        // The header needs to be represented by snake case
        dumps['mock_info']['header_deps'].push({
            timestamp: header.timestamp,
            number: header.number,
            epoch: header.epoch,
            compact_target: header.compactTarget,
            dao: header.dao,
            hash: header.hash,
            nonce: header.nonce,
            parent_hash: header.parentHash,
            proposals_hash: header.proposalsHash,
            transactions_root: header.transactionsRoot,
            extra_hash: header.extraHash,
            version: header.version,
        });
    }

    writeFile(path, JSON.stringify(dumps, null, 4), (err) => {
        if (err) throw err;
        console.log(`tx has been saved at ${path}`);
    })
}
