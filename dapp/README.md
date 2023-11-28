# Spore Typed Message

## Create and transfer Spore on test network by default secp256k1 lock

```shell
$ npm install

$ npm run skCreateSpore
# Replace TxHash in src/skTransferSpore.ts
$ npm run skTransferSpore
```

## Create and transfer Spore on dev network by typed message

```shell
$ npm run tmCreateSpore
# Replace TxHash in src/tmTransferSpore.ts
$ npm run tmTransferSpore
```

## Create and transfer Spore on test network by typed message

Open `src/tmConfig.ts` and then switch configurations by switching comments.

Real examples: <https://pudge.explorer.nervos.org/transaction/0xd6bc4b242d9e24947ca6c50982f4fdc61f2b7d32ca92775b8c4ac3d1b4c665fd>

```shell
$ npm run tmCreateSpore
# Replace TxHash in src/tmTransferSpore.ts
$ npm run tmTransferSpore
```
