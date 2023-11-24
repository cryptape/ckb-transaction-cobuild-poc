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
    { infoHash: blockchain.Byte32, data: blockchain.Bytes },
    ["infoHash", "data"]
);

export const ScriptAction = table(
    { scriptHash: blockchain.Byte32, action: Action },
    ["scriptHash", "action"],
)

export const TypedMessageV1 = vector(ScriptAction);

export const TypedMessage = union(
    { TypedMessageV1 },
    ["TypedMessageV1"]
)

export const SighashWithAction = table(
    { lock: blockchain.Bytes, message: TypedMessage },
    ["lock", "message"]
)

export const Sighash = table(
    { lock: blockchain.Bytes },
    ["lock"]
)

export const DappInfoV1 = table(
    {
        name: String,
        url: String,
        scriptHash: blockchain.Byte32,
        schema: String,
        messageType: String,
    },
    ["name", "url", "scriptHash", "schema", "messageType"]
)

export const DappInfo = union(
    { DappInfoV1 },
    ["DappInfoV1"]
)

export const DappInfos = vector(DappInfo);

export const Scratch = union(
    { ScriptAction },
    ["ScriptAction"]
)

export const ScratchOpt = option(Scratch);

export const SigningAction = table(
    {
        flags: Uint64,
        address: Address,
        message: TypedMessage,
        skeletonHash: blockchain.Byte32,
        infos: DappInfos,
        scratch: ScratchOpt,
    },
    ["flags", "address", "message", "skeletonHash", "infos", "scratch"]
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
        message: TypedMessage,
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
