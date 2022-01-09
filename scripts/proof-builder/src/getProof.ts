import { ApiPromise, WsProvider } from "@polkadot/api";
import { TypeRegistry } from "@polkadot/types";
import { xxhashAsU8a, blake2AsU8a } from "@polkadot/util-crypto";
import {
  u8aToHex,
  hexToU8a,
  hexToBn,
  bnToHex,
  bnToU8a,
  BN,
} from "@polkadot/util";
import { Keyring } from "@polkadot/api";

// Our address for Alice on the dev chain
const ALICE = "hJKzPoi3MQnSLvbShxeDmzbtHncrMXe5zwS3Wa36P6kXeNpcv";
const BOB = "hJJREXdsDH4SSmaxCXiFqFFyKgmqU56gfSbuBWfxi2mcQbfvj";


// Resources 
// https://www.shawntabrizi.com/assets/presentations/substrate-storage-deep-dive.pdf


// In substrate data is stored in a trie and can be accessed by navigating the 
// path to a specific value.
// 
// Path construction
// xxhashAsU8a Pallet Name  - Assets
// xxhashAsU8a Storage Name - Account
// blake2AsU8a AssetId 32B  - H(102)
// Raw         AssetId 32B  - 102 (for tether dollar on staging)
// blake2AsU8a SS58         - H(hJKzP..)
// Raw         SS58         - hJKzP.. 
function createTrieKeyPath(assetId: any, account: any) {

  // prepare the storage prefix key for system events.
  let module_hash = xxhashAsU8a("Assets", 128);
  let storage_value_hash = xxhashAsU8a("Account", 128);

  // syntax to concatenate Uint8Array
  let prefixKey = new Uint8Array([...module_hash, ...storage_value_hash]);

  // we need a keyring to decode the ss58 account
  const keyring = new Keyring();

  // convert asset id to a 32B little endian array
  let assetIdBytes = new Uint8Array([...new BN(assetId).toArray("le", 4)]);

  // convert accoubt into bytes with keyring
  let accountBytes = keyring.decodeAddress(account);

  // lets prepare the hashed suffix values
  let moduleNameBlake = blake2AsU8a(assetIdBytes, 128);
  let accountBytesBlake = blake2AsU8a(accountBytes, 128);

  // Special syntax to concatenate Uint8Array
  let suffixKey = new Uint8Array([
    ...moduleNameBlake,
    ...assetIdBytes,
    ...accountBytesBlake,
    ...accountBytes,
  ]);

  // concat byte arrays
  let result = new Uint8Array([...prefixKey, ...suffixKey]);
  return u8aToHex(result);
}

// We can fetch a users Assets Account balance at a specific block height

async function main() {
  // Construct
  const wsProvider = new WsProvider("wss://staging-rpc.parallel.fi/");
  const api = await ApiPromise.create({ provider: wsProvider });

  // let key = createTrieKeyPath(102, ALICE);
  let key = createTrieKeyPath(102, BOB);

  // see: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fstaging-rpc.parallel.fi%2F#/explorer
  //
  // let at = "0x2b84124ab1b71a23c1099fde7800d32eb219061db6248e57fd3efc454fb51d0a"; // 31,736
  let at = "0xe37b7ea2f68c948435b5515758710d5408b72d9f12ed439202b7e389592cf042"; // 36,516

  // get storage object as a hex string we'll manually parse
  let value = await api.rpc.state.getStorage(key, at);

  const registry = new TypeRegistry();
  const parsedValue = registry.createType("Text", value);
  const assetBalance = parsedValue.toJSON()

  // fetch proof at height
  let proof = await api.rpc.state.getReadProof([key], at);

  // convert the first 32B to a le big number - we add 2 for 0x
  let readableTokenAmount = hexToBn(assetBalance.slice(0, 32 + 2), { isLe: true }).toString()

  // output
  console.log({
    key,
    at,
    assetBalance,
    readableTokenAmount
  })

  console.log(proof.toJSON());
}

// run
main()
  .catch(console.error)
  .finally(() => process.exit());