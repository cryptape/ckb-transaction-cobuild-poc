# ckb-typed-message(PoC)
This project is a proof of concept that aims to demonstrate how to adopt typed
messages (similar to EIP-712) in CKB scripts. It also includes extended
witnesses to simplify signing and DApp interoperability.


## Build
Build contracts:

``` sh
capsule build
```

Run tests:

``` sh
cd tests && cargo test
```

## Project Structure
* ckb-typed-message

A library for writing scripts with typed message support.
* contracts/typed-message-lock-demo

A demo lock demonstrating how to write a lock script with typed message support.
* dapp

DApp and wallet demo projects. With these projects, we can test/deploy on the testnet.

* schemas

The molecule definitions
* tests

uint tests
