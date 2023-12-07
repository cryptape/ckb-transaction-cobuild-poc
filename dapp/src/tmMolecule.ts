import { blockchain } from '@ckb-lumos/base';
import {
    molecule,
    number,
} from "@ckb-lumos/codec";
const { Uint64, Uint32 } = number;
const { option, table, vector, union } = molecule;
const { Script } = blockchain;

export const String = blockchain.Bytes;

export const Address = union(
    { Script },
    ["Script"]
);

export const AddressOpt = option(Address);

export const Action = table(
    {
        scriptInfoHash: blockchain.Byte32,
        scriptHash: blockchain.Byte32,
        data: blockchain.Bytes
    },
    ["scriptInfoHash", "scriptHash", "data"]
);

export const ActionVec = vector(Action);

export const Message = table(
    { actions: ActionVec },
    ["actions"]
)

export const ScriptInfo = table(
    {
        name: String,
        url: String,
        scriptHash: blockchain.Byte32,
        schema: String,
        messageType: String,
    },
    ["name", "url", "scriptHash", "schema", "messageType"]
)

export const ScriptInfoVec = vector(ScriptInfo);

export const BuildingPacketV1 = table(
    {
        message: Message,
        payload: blockchain.Transaction,
        scriptInfos: ScriptInfoVec,
        lockActions: ActionVec,
    },
    ["message", "payload", "scriptInfos", "lockActions"]
)

export const BuildingPacket = union(
    { BuildingPacketV1 },
    ["BuildingPacketV1"]
)

export const SighashAll = table(
    { seal: blockchain.Bytes, message: Message },
    ["seal", "message"]
)

export const SighashOnly = table(
    { seal: blockchain.Bytes },
    ["seal"]
)

export const OtxStart = table(
    {
        startInputCell: Uint32,
        startOutputCell: Uint32,
        startCellDeps: Uint32,
        startHeaderDeps: Uint32,
    },
    ["startInputCell", "startOutputCell", "startCellDeps", "startHeaderDeps"]
)

export const Otx = table(
    {
        lock: blockchain.Bytes,
        inputCells: Uint32,
        outputCells: Uint32,
        cellDeps: Uint32,
        headerDeps: Uint32,
        message: Message,
    },
    ["lock", "inputCells", "outputCells", "cellDeps", "headerDeps", "message"]
)

export const Mint = table(
    { id: blockchain.Byte32, to: Address, contentHash: blockchain.Byte32 },
    ["id", "to", "contentHash"]
)

export const Transfer = table(
    { nftID: blockchain.Byte32, from: AddressOpt, to: AddressOpt },
    ["nftID", "from", "to"]
)

export const Melt = table(
    { id: blockchain.Byte32 },
    ["id"]
)

export const SporeAction = union(
    { Mint, Transfer, Melt },
    ["Mint", "Transfer", "Melt"]
)
