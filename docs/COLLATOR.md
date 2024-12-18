# How to run collators for parallel?

Parallel/Heiko network will open collators to the community once the chain becomes stable for some days. You will be able to run collators
using the following methods.

## Method One - Governance (not advised, mostly for Parallel Team)

You'll need to submit a motion first to increase the default invulnerables set. If the council agreed on this motion, you can then
start to run your collators.

The steps are as following:

1. Submit motion, `collatorSelection -> setinvulnerables` and add your collator's account id.

2. Generate collator's keystore using key command

```
./target/debug/parallel key insert -d . --keystore-path keystore --key-type aura
```

3. Launch collator

```
./scripts/collator.sh <NODE_KEY> <KEYSTORE_PATH> <TELEMETRY_DISPLAY_NAME>
```

4. Set session keys

You'll need to prepare your collator's Sr25519 public key, and then use it to set collator's session keys

```
subkey inspect <SURL> --scheme Sr25519
```

Then connect to polkadot.js, add use collator account to sign the following extrinsic:

```
Extrinsics -> session -> setKeys(sr25519_pubkey, 0x1234)
```

5. Wait the next session (6 hours)

If everything has been done successfully, your collator will be able to start producing blocks in around 6 hours (the next session)

## Method Two - Register as candidate

Before registering yourself as collator candidate, you'll need to prepare enough HKO, a fixed number of HKO will be locked util
that you get kicked for not producing blocks.

The steps are as following:

1. Prepare HKO

2. Launch collator and insert key

```
curl http://localhost:9944 -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "author_insertKey",
    "params": ["<aura/gran>", "<mnemonic phrase>", "<public key>"]
}'
```

3. Connect to collator using polkadot.js

4. Rotate keys

```
Developer -> RPC calls -> author -> rotateKeys
```

5. Set session keys using collator account

```
Developer -> Extrinsics -> session -> setKeys(sr25519_pubkey, 0x1234)
```

6. Register as collator candidate

```
Developer -> Extrinsics -> collatorSelection -> registerAsCandidate
```

7. Wait the next session (6 hours)
