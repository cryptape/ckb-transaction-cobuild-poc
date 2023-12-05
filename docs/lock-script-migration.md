
This document describes the migration process from an existing lock script to
one that supports typed messages. The migration process involves three parts:
the lock script (smart contract), DApp, and wallet.

## Lock Script Changes
The changes to the lock script are relatively simple. A TL;DR version is to only add one
line of code to existing project(See
[example](https://github.com/XuJiandong/ckb-typed-message-poc/blob/24e764ed01c29cbf5be17225402f4847a6f50992/contracts/typed-message-lock-demo/src/entry.rs#L20)):
```Rust
let (digest_message, lock) = parse_typed_message()?;
```
After making this change, the parsed values will be used in the signature
validation process.

In the previous implementation of the lock script, the digest_message and lock
values were calculated or parsed from the transaction hash and witness. You can
refer to the [system
script](https://github.com/nervosnetwork/ckb-system-scripts/blob/master/c/secp256k1_blake160_sighash_all.c)
for more details on how these values were derived.

With the added support for typed messages, the digest message is calculated using the following components:
- skeleton hash
- typed message

The skeleton hash is calculated using the following components:
- transaction hash
- witnesses with index beyond input cell length

The final digest message can make the following parts not malleable:
- transaction
- witnesses used by all type scripts
- extract witnesses with index beyond input cell length

The `lock` value is actually the same as before, it is located in
[SighashWithAction](https://github.com/XuJiandong/ckb-typed-message-poc/blob/24e764ed01c29cbf5be17225402f4847a6f50992/schemas/basic.mol#L28).

There is only one `SighashWithAction` variant in witness in whole transaction. If
there are some other locks with typed message, they should use `Sighash` variant.
As designed, the `Sighash` variant doesn't include typed message part.

## Dapp Changes

Firstly, we need to define some actions for the Dapp. For example, if the Dapp is an NFT application, it should include several action types: Mint, Transfer, or Melt. We can pass some parameters in these actions to indicate "who created this NFT," "to whom this NFT should be transferred," etc. The relevant definition file can be found [here](https://github.com/cryptape/ckb-typed-message-poc/blob/main/schemas/spore.mol). The definition file should be public, accessible to anyone.

When a user attempts a certain action, such as transferring an NFT to another person, the Dapp should send the constructed `Transaction` and `SigningAction` together to the wallet. The construction of the SigningAction is relatively complex and can be referenced in the diagram below.

The detailed steps can be found by reading our POC code.

```
                                                                             ┌────────────────────┐
                                                                             │                    │
                                                                             │    SigningAction   │
              ┌───────────────┐                                              ├────────────────────┤
              │    DappInfo   │                                              │                    │
              ├───────────────┤                                              │  Flags             │
              │ Name          │                                              │  (Always 0)        │
┌───────────┐ │               │                                              │                    │
│  schema   │ │ Url           │   ┌───────────────┐     ┌────────────────┐   │  Address           │
├───────────┤ │               │   │    Action     │     │  ScriptAction  │   │  (Lock script)     │
│  Mint     │ │ ScriptHash    │   ├───────────────┤     ├────────────────┤   │                    │
│           │ │               │   │               │     │                ├───┼─> Message          │
│  Transfer ├─┼─>Schema       ├───┼─>DappInfoHash ├─────┼─>Action        │   │                    │
│           │ │               │   │               │     │                │   │  SkeletonHash      │
│  Melt     │ │ MessageType   │   │               │     │                │   │                    │
└───────────┘ └───────────────┘   │               │     │                │   │  Infos             │
                                  │               │     │ ScriptHash     │   │  (Raw data of      │
┌────────────────────────────┐    │               │     │ (Dapp's type   │   │   DappInfo)        │
│ Transfer my NFT to jack!   ├────┼─>Data         │     │  script hash)  │   │                    │
│                            │    │               │     │                │   │  Scratch           │
└────────────────────────────┘    └───────────────┘     └────────────────┘   └────────────────────┘
```

## Wallet Changes

When the wallet receives the Transaction and SigningAction, it should verify the following information:

1. Check if the DappInfoHash in each Action corresponds to the DappInfo.
2. Verify the SkeletonHash.
3. Display the Message on the screen and wait for the user to click the confirm button.
4. Calculate digest_message: digest_message = hash(skeleton_hash + message_len + message), where message_len is represented in 64-bit little-endian format.
5. Combine digest_message and lock into SighashWithAction, serialize it, and write it into the corresponding witness.

## Others
The type script changes are not covered in this document.
