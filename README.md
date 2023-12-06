# CKB Transaction Cooperate Build(PoC)
This project is a proof of concept that aims to demonstrate how to adopt
transaction cooperate build and message (similar to EIP-712) in CKB. It also
includes witnesses layout change to simplify signing and DApp interoperability.


## Build
Build contracts:

```sh
capsule build --release
```

## Integration with Dapp and Wallet
See [dapp](./dapp/README.md), using Lumos and Spore SDK.


## Project Structure
* ckb-transaction-cobuild

    A library for writing scripts

* contracts/transaction-cobuild-lock-demo

    A demo lock demonstrating how to write a lock script

* dapp

    DApp and wallet demo projects. With these projects, we can test/deploy on the testnet/devnet.

* schemas

    The molecule definitions

* tests

    uint tests
