This tool builds storage proofs for an Asset Account at a specific blockhash

```bash
npx ts-node src/getProof.ts
```

output
```json
{
  "key": "0x682a5...",
  "at": "0xe37b7...",
  "assetBalance": "0x80588...",
  "readableTokenAmount": "1234000000"
}
{
  "at": "0xe37b7...",
  "proof": [
    "0x9e2a5...",
    "0x80200...",
    "0x9e9d8...",
    "0x80006...",
    "0x80fff...",
    "0xa700f...",
    "0x7f200..."
  ]
}
```