
This demo lock is based on [SECP256K1/blake160](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md#secp256k1blake160) with typed message support.
The [ckb-auth](https://github.com/nervosnetwork/ckb-auth) (commit: cdc2c52) is used to simplify authentication.

### Script

```yaml
code_hash: <code hash>
hash_type: <hash type>
args: <pubkey blake160 hash, 20 bytes>
```

### Witness

```yaml
witness: ExtendedWitness format, SighashWithAction variant
    lock: <secp256k1 signature, 65 bytes>
    message: <typed message>
```
or

```yaml
witness: ExtendedWitness format, Sighash variant
    lock: <secp256k1 signature, 65 bytes>
```
