import { createConfig } from '@ckb-lumos/config-manager';
import { HashType, DepType } from '@ckb-lumos/base';
import { SporeConfig, PredefinedSporeConfigScriptName } from '@spore-sdk/core';

// export const config: SporeConfig<PredefinedSporeConfigScriptName> = {
//     lumos: createConfig(createConfig({
//         PREFIX: "ckt",
//         SCRIPTS: {
//             SECP256K1_BLAKE160: {
//                 CODE_HASH:
//                     "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
//                 HASH_TYPE: "type",
//                 TX_HASH:
//                     "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
//                 INDEX: "0x0",
//                 DEP_TYPE: "depGroup",
//                 SHORT_ID: 0,
//             },
//             SECP256K1_BLAKE160_MULTISIG: {
//                 CODE_HASH:
//                     "0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8",
//                 HASH_TYPE: "type",
//                 TX_HASH:
//                     "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
//                 INDEX: "0x1",
//                 DEP_TYPE: "depGroup",
//                 SHORT_ID: 1,
//             },
//             DAO: {
//                 CODE_HASH:
//                     "0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e",
//                 HASH_TYPE: "type",
//                 TX_HASH:
//                     "0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f",
//                 INDEX: "0x2",
//                 DEP_TYPE: "code",
//             },
//             SUDT: {
//                 CODE_HASH:
//                     "0xc5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4",
//                 HASH_TYPE: "type",
//                 TX_HASH:
//                     "0xe12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769",
//                 INDEX: "0x0",
//                 DEP_TYPE: "code",
//             },
//             ANYONE_CAN_PAY: {
//                 CODE_HASH:
//                     "0x3419a1c09eb2567f6552ee7a8ecffd64155cffe0f1796e6e61ec088d740c1356",
//                 HASH_TYPE: "type",
//                 TX_HASH:
//                     "0xec26b0f85ed839ece5f11c4c4e837ec359f5adc4420410f6453b1f6b60fb96a6",
//                 INDEX: "0x0",
//                 DEP_TYPE: "depGroup",
//                 SHORT_ID: 2,
//             },
//             OMNILOCK: {
//                 CODE_HASH:
//                     "0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb",
//                 HASH_TYPE: "type",
//                 TX_HASH:
//                     "0x27b62d8be8ed80b9f56ee0fe41355becdb6f6a40aeba82d3900434f43b1c8b60",
//                 INDEX: "0x0",
//                 DEP_TYPE: "code",
//             },
//         },
//     })),
//     ckbNodeUrl: 'https://testnet.ckb.dev/rpc',
//     ckbIndexerUrl: 'https://testnet.ckb.dev/indexer',
//     maxTransactionSize: 500 * 1024,
//     scripts: {
//         Spore: {
//             script: {
//                 codeHash: '0xbbad126377d45f90a8ee120da988a2d7332c78ba8fd679aab478a19d6c133494',
//                 hashType: 'data1',
//             },
//             cellDep: {
//                 outPoint: {
//                     txHash: '0xfd694382e621f175ddf81ce91ce2ecf8bfc027d53d7d31b8438f7d26fc37fd19',
//                     index: '0x0',
//                 },
//                 depType: 'code',
//             },
//             versions: [],
//         },
//         Cluster: {
//             script: {
//                 codeHash: '0x598d793defef36e2eeba54a9b45130e4ca92822e1d193671f490950c3b856080',
//                 hashType: 'data1',
//             },
//             cellDep: {
//                 outPoint: {
//                     txHash: '0x49551a20dfe39231e7db49431d26c9c08ceec96a29024eef3acc936deeb2ca76',
//                     index: '0x0',
//                 },
//                 depType: 'code',
//             },
//             versions: [],
//         },
//     },
//     extensions: [],
// };


export const config: SporeConfig<PredefinedSporeConfigScriptName> = {
    lumos: createConfig(createConfig({
        PREFIX: "ckt",
        SCRIPTS: {
            SECP256K1_BLAKE160: {
                CODE_HASH:
                    "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                HASH_TYPE: "type",
                TX_HASH:
                    "0x94dfbac7f4ccb5b1f41b1cc314abae9df3f85108af214c5aaff569c8ada0885e",
                INDEX: "0x0",
                DEP_TYPE: "depGroup",
                SHORT_ID: 0,
            },
        },
    })),
    ckbNodeUrl: 'http://18.162.168.78:8114/rpc',
    ckbIndexerUrl: 'http://18.162.168.78:8114/indexer',
    maxTransactionSize: 500 * 1024,
    scripts: {
        Spore: {
            script: {
                codeHash: '0xbbad126377d45f90a8ee120da988a2d7332c78ba8fd679aab478a19d6c133494',
                hashType: 'data1',
            },
            cellDep: {
                outPoint: {
                    txHash: '0x769d127f11d81e8f6e8aefd45ea4fe8a32e9d84e70ec1267170c6db82b9796f0',
                    index: '0x0',
                },
                depType: 'code',
            },
            versions: [],
        },
        Cluster: {
            script: {
                codeHash: '0x598d793defef36e2eeba54a9b45130e4ca92822e1d193671f490950c3b856080',
                hashType: 'data1',
            },
            cellDep: {
                outPoint: {
                    txHash: '0x769d127f11d81e8f6e8aefd45ea4fe8a32e9d84e70ec1267170c6db82b9796f0',
                    index: '0x1',
                },
                depType: 'code',
            },
            versions: [],
        },
    },
    extensions: [],
};

export const configAuth = {
    script: {
        codeHash: '0x9017dadb5493e6317da3bab8a1456851d45043ff701d6455a03abdabcad99e3e',
        hashType: 'data1' as HashType,
    },
    cellDep: {
        outPoint: {
            txHash: '0x27ce6f8e9032d40334948904b5a769587eaf0d8e79e1b51a80c9ba0616e102f2',
            index: '0x0',
        },
        depType: 'code' as DepType,
    }
}

export const configTypedMessageLockDemo = {
    script: {
        codeHash: '0x81046990df3542a0563555af6b863fa4ec8d1d60d77ed8d654d981c7c015f6b2',
        hashType: 'data1' as HashType,
    },
    cellDep: {
        outPoint: {
            txHash: '0x068660a7f5d97f6e4adf34983c5223a02b2caf0bc139d083ab9244c4dc309da9',
            index: '0x0',
        },
        depType: 'code' as DepType,
    }
}
