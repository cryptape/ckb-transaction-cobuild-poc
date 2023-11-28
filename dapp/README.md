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

```shell
$ npm run tmCreateSpore
# Replace TxHash in src/tmTransferSpore.ts
$ npm run tmTransferSpore
```
