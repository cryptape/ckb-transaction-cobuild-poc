import {
    Cell,
    CellDep,
    HexString,
    PackedSince,
    WitnessArgs,
    blockchain,
    values,
} from "@ckb-lumos/base";
import { bytes } from "@ckb-lumos/codec";
import { FromInfo } from "@ckb-lumos/common-scripts";
import {
    Options,
    TransactionSkeletonType,
} from "@ckb-lumos/helpers";
import { configAuth, config as configLumos, configTypedMessageLockDemo } from './tmConfig';
const { ScriptValue } = values;

export const SECP_SIGNATURE_PLACEHOLDER = "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

export function addCellDep(
    txSkeleton: TransactionSkeletonType,
    newCellDep: CellDep
): TransactionSkeletonType {
    const cellDep = txSkeleton.get("cellDeps").find((cellDep) => {
        return (
            cellDep.depType === newCellDep.depType &&
            new values.OutPointValue(cellDep.outPoint, { validate: false }).equals(
                new values.OutPointValue(newCellDep.outPoint, { validate: false })
            )
        );
    });

    if (!cellDep) {
        txSkeleton = txSkeleton.update("cellDeps", (cellDeps) => {
            return cellDeps.push({
                outPoint: newCellDep.outPoint,
                depType: newCellDep.depType,
            });
        });
    }

    return txSkeleton;
}

export async function setupInputCell(
    txSkeleton: TransactionSkeletonType,
    inputCell: Cell,
    _fromInfo?: FromInfo,
    {
        config = undefined,
        defaultWitness = "0x",
        since = undefined,
    }: Options & {
        defaultWitness?: HexString;
        since?: PackedSince;
    } = {}
): Promise<TransactionSkeletonType> {
    config = configLumos.lumos
    const fromScript = inputCell.cellOutput.lock;
    // if (!isSecp256k1Blake160Script(fromScript, config)) {
    //   throw new Error(`Not SECP256K1_BLAKE160 input!`);
    // }

    // add inputCell to txSkeleton
    txSkeleton = txSkeleton.update("inputs", (inputs) => {
        return inputs.push(inputCell);
    });

    const output: Cell = {
        cellOutput: {
            capacity: inputCell.cellOutput.capacity,
            lock: inputCell.cellOutput.lock,
            type: inputCell.cellOutput.type,
        },
        data: inputCell.data,
    };

    txSkeleton = txSkeleton.update("outputs", (outputs) => {
        return outputs.push(output);
    });

    if (since) {
        txSkeleton = txSkeleton.update("inputSinces", (inputSinces) => {
            return inputSinces.set(txSkeleton.get("inputs").size - 1, since);
        });
    }

    txSkeleton = txSkeleton.update("witnesses", (witnesses) => {
        return witnesses.push(defaultWitness);
    });

    // const template = config.SCRIPTS.SECP256K1_BLAKE160;
    // if (!template) {
    //     throw new Error(`SECP256K1_BLAKE160 script not defined in config!`);
    // }

    // const scriptOutPoint: OutPoint = {
    //     txHash: configTypedMessageLockDemo.cellDep.outPoint.txHash,
    //     index: template.INDEX,
    // };

    // add cell dep
    txSkeleton = addCellDep(txSkeleton, {
        outPoint: {
            txHash: config.SCRIPTS.SECP256K1_BLAKE160.TX_HASH,
            index: config.SCRIPTS.SECP256K1_BLAKE160.INDEX,
        },
        depType: config.SCRIPTS.SECP256K1_BLAKE160.DEP_TYPE,
    })
    txSkeleton = addCellDep(txSkeleton, {
        outPoint: configAuth.cellDep.outPoint,
        depType: configAuth.cellDep.depType,
    });
    txSkeleton = addCellDep(txSkeleton, {
        outPoint: configTypedMessageLockDemo.cellDep.outPoint,
        depType: configTypedMessageLockDemo.cellDep.depType,
    });

    // add witness
    /*
     * Modify the skeleton, so the first witness of the fromAddress script group
     * has a WitnessArgs construct with 65-byte zero filled values. While this
     * is not required, it helps in transaction fee estimation.
     */
    const firstIndex = txSkeleton
        .get("inputs")
        .findIndex((input) =>
            new ScriptValue(input.cellOutput.lock, { validate: false }).equals(
                new ScriptValue(fromScript, { validate: false })
            )
        );
    if (firstIndex !== -1) {
        while (firstIndex >= txSkeleton.get("witnesses").size) {
            txSkeleton = txSkeleton.update("witnesses", (witnesses) =>
                witnesses.push("0x")
            );
        }
        let witness: string = txSkeleton.get("witnesses").get(firstIndex)!;
        const newWitnessArgs: WitnessArgs = {
            /* 65-byte zeros in hex */
            lock: SECP_SIGNATURE_PLACEHOLDER,
        };
        if (witness !== "0x") {
            const witnessArgs = blockchain.WitnessArgs.unpack(bytes.bytify(witness));
            const lock = witnessArgs.lock;
            if (
                !!lock &&
                !!newWitnessArgs.lock &&
                !bytes.equal(lock, newWitnessArgs.lock)
            ) {
                throw new Error(
                    "Lock field in first witness is set aside for signature!"
                );
            }
            const inputType = witnessArgs.inputType;
            if (inputType) {
                newWitnessArgs.inputType = inputType;
            }
            const outputType = witnessArgs.outputType;
            if (outputType) {
                newWitnessArgs.outputType = outputType;
            }
        }
        witness = bytes.hexify(blockchain.WitnessArgs.pack(newWitnessArgs));
        txSkeleton = txSkeleton.update("witnesses", (witnesses) =>
            witnesses.set(firstIndex, witness)
        );
    }

    return txSkeleton;
}
