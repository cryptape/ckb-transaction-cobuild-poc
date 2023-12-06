
This document describes the migration process from an existing lock script to
one that supports typed messages. The migration process involves three parts:
the lock script (smart contract), DApp, and wallet.

## Lock Script Changes
The changes to the lock script are relatively simple. A TL;DR version is to only add one
line of code to existing project(See
[example](https://github.com/cryptape/ckb-typed-message-poc/blob/76676d0b229c914743b0204931b98f4c8e4e71e6/contracts/typed-message-lock-demo/src/entry.rs#L20)):
```Rust
let (message_digest, seal) = parse_message()?;
```
After making this change, the parsed values will be used in the signature
validation process.

In the previous implementation of the lock script, the message_digest and lock
values were calculated or parsed from the transaction hash and witness. You can
refer to the [system
script](https://github.com/nervosnetwork/ckb-system-scripts/blob/master/c/secp256k1_blake160_sighash_all.c)
for more details on how these values were derived.

With the added support for typed messages, the message digest is calculated using the following components:
- skeleton hash
- typed message

The [skeleton hash]((https://github.com/cryptape/ckb-typed-message-poc/blob/76676d0b229c914743b0204931b98f4c8e4e71e6/ckb-typed-message/src/lib.rs#L112)) is calculated using the following components:
- transaction hash
- witnesses with index beyond input cell length

The final message digest can make the following parts not malleable:
- transaction
- witnesses used by all type scripts
- extract witnesses with index beyond input cell length

The `lock` value is actually the same as before, it is located in
[SighashWithAction](https://github.com/XuJiandong/ckb-typed-message-poc/blob/24e764ed01c29cbf5be17225402f4847a6f50992/schemas/basic.mol#L28).

There is only one `SighashWithAction` variant in witness in whole transaction. If
there are some other locks with typed message, they should use `Sighash` variant.
As designed, the `Sighash` variant doesn't include typed message part.

## Dapp Changes

Firstly, we need to define some actions for the Dapp. For example, if the Dapp
is an NFT application, it should include several action types: Mint, Transfer,
or Melt. We can pass some parameters in these actions to indicate "who created
this NFT," "to whom this NFT should be transferred," etc. An example definition
file can be found
[here](https://github.com/cryptape/ckb-typed-message-poc/blob/main/schemas/spore.mol).
The definition file should be public, accessible to anyone.

When a user attempts a certain action, such as transferring an NFT to another
person, the Dapp should send the constructed `SigningAction` to the wallet. The
construction of the SigningAction is relatively complex and can be referenced in
the diagram below.

The detailed steps can be found by reading [the
code](https://github.com/cryptape/ckb-typed-message-poc/blob/main/dapp/src/tmTransferSpore.ts).

```
                                                                             ┌────────────────────┐
                                                                             │    SigningAction   │
              ┌───────────────┐                                              ├────────────────────┤
              │    DappInfo   │                                              │                    │
              ├───────────────┤                                              │  Flags             │
              │  Name         │                                              │  (Always 0)        │
┌───────────┐ │               │                                              │                    │
│  schema   │ │  Url          │   ┌───────────────┐     ┌────────────────┐   │  Address           │
├───────────┤ │               │   │    Action     │     │  ScriptAction  │   │  (Lock script)     │
│  Mint     │ │  ScriptHash   │   ├───────────────┤     ├────────────────┤   │                    │
│           │ │               │   │               │     │                ├───┼─>Message           │
│  Transfer ├─┼─>Schema       ├───┼─>DappInfoHash ├─────┼─>Action        │   │                    │
│           │ │               │   │               │     │                │   │  SkeletonHash      │
│  Melt     │ │  MessageType  │   │               │     │                │   │                    │
└───────────┘ └───────────────┘   │               │     │                │   │  Infos             │
                                  │               │     │  ScriptHash    │   │  (Raw data of      │
┌────────────────────────────┐    │               │     │  (Dapp's type  │   │   DappInfo)        │
│ Transfer my NFT to jack!   ├────┼─>Data         │     │  script hash)  │   │                    │
│                            │    │               │     │                │   │  Scratch           │
└────────────────────────────┘    └───────────────┘     └────────────────┘   └────────────────────┘
```

Note that the Message field in the SigningAction is referred to as the typed
message. This typed message may vary depending on the DApp. It is constructed by
the DApp and displayed on the wallet. From the perspective of the lock script,
the typed message is treated as a black box.


The DApp will receive a signature if the wallet users approve and sign it. After
that, the DApp can fill the signature in the lock field in SighashWithAction and
broadcast the transaction to the CKB p2p network.

In most scenarios, the size of the SigningAction is significantly smaller than
the transaction itself. Therefore, it is not necessary to send the entire
transaction along with the SigningAction. The SigningAction contains sufficient
information, making it easy to implement Wallet RPC/API functionality.

## Wallet Changes

When the wallet receives the SigningAction, it should perform [the following steps](https://github.com/cryptape/ckb-typed-message-poc/blob/main/dapp/src/tmWallet.ts):

- Verify if the DappInfoHash in each Action corresponds to the DappInfo.
- Display the Typed Message and DappInfo on the screen and wait for the user to confirm.
- When the user clicks the confirm button, calculate the message digest based on
  the skeleton hash and typed message and sign it.
- Send the signature back to the DApp.


## Others

The changes to the type script are not covered in this document. Essentially,
the type script should parse the typed message and verify that the information
contained within it is true, according to the transaction.
