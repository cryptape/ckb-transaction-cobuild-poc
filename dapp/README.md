# Spore with Transaction Co-Build Protocol

## Create and transfer Spore on dev network

```shell
$ npm run tmCreateSpore
# Replace TxHash in src/tmTransferSpore.ts
$ npm run tmTransferSpore
```

The message will be printed out before signing. In a real-life project, a
popup window should be displayed to users to show these messages.

You can also use the WebUI to view the Spores you own and transfer it by

```shell
$ npm run web
```

## Create and transfer Spore on testnet

Open `src/tmConfig.ts` and then switch configurations by switching comments.

Real examples: <https://pudge.explorer.nervos.org/transaction/0xd6bc4b242d9e24947ca6c50982f4fdc61f2b7d32ca92775b8c4ac3d1b4c665fd>

```shell
$ npm run tmCreateSpore
# Replace TxHash in src/tmTransferSpore.ts
$ npm run tmTransferSpore
```
