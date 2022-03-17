## Runtime Upgrade Guide

### Checklist

- [ ] Spec_version, transaction_version is bumped
- [ ] Previous storage migration should have been removed from runtime's OnRuntimeUpgrade impl
- [ ] New storage migration should have been added to runtime
- [ ] Node should be bumped first. After starting to produce blocks in a stable way we can then bump runtime
- [ ] Srtool compiled runtime wasm should be preferred, compressed runtime wasm is preferred
- [ ] Runtime wasm should be <= 1MB, if it's a bit more than 1MB it's ok but shouldn't excess too much
- [ ] Live.md should be updated after the runtime upgrade

### via Sudo

```
sudo -> sudo -> parachainSystem -> authorizeUpgrade
sudo -> sudo -> parachainSystem -> enactUpgrade
```

### via Council (council member only)

1. propose motion
2. vote for your motion
3. ask other council members to vote
4. close and execute the motion
