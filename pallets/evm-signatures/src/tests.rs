// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate as evm_signatures;
use codec::Encode;
use evm_signatures::*;
use frame_support::{assert_err, assert_ok, parameter_types, traits::AsEnsureOriginWithArg};
use frame_system::{EnsureRoot, EnsureSigned};
use hex_literal::hex;
use sp_core::{ecdsa, Pair};
use sp_io::hashing::keccak_256;
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::{
    testing::{Header, H256},
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    transaction_validity::TransactionPriority,
    MultiSignature, MultiSigner,
};

pub const ECDSA_SEED: [u8; 32] =
    hex_literal::hex!["7e9c7ad85df5cdc88659f53e06fb2eb9bab3ebc59083a3190eaf2c730332529c"];

type Balance = u128;
type BlockNumber = u64;
type CurrencyId = u32;
type Signature = MultiSignature;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type Block = frame_system::mocking::MockBlock<Runtime>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;

frame_support::construct_runtime!(
    pub enum Runtime where
       Block = Block,
       NodeBlock = Block,
       UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances,
        System: frame_system,
        Assets: pallet_assets,
        EVMSignatures: evm_signatures,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type BaseCallFilter = frame_support::traits::Everything;
    type Index = u32;
    type BlockNumber = BlockNumber;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type SystemWeightInfo = ();
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Pallet<Runtime>;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

parameter_types! {
    pub const AssetDeposit: u64 = 1;
    pub const ApprovalDeposit: u64 = 1;
    pub const AssetAccountDeposit: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = CurrencyId;
    type AssetIdParameter = codec::Compact<CurrencyId>;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type AssetAccountDeposit = AssetAccountDeposit;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
    type RemoveItemsLimit = frame_support::traits::ConstU32<1000>;
    type CallbackHandle = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

parameter_types! {
    pub const Priority: TransactionPriority = TransactionPriority::MAX;
    pub const CallFee: Balance = 42;
    pub const CallMagicNumber: u16 = 0xff50;
    pub const NativeCurrencyId: CurrencyId = 0;
    pub const VerifySignature: bool = true;
}

impl Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Signature = ethereum::EthereumSignature;
    type Signer = <Signature as Verify>::Signer;
    type CallMagicNumber = CallMagicNumber;
    type Currency = Balances;
    type CallFee = CallFee;
    type OnChargeTransaction = ();
    type UnsignedPriority = Priority;
    type GetNativeCurrencyId = NativeCurrencyId;
    type VerifySignature = VerifySignature;
    type Assets = Assets;
    type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
    type WithdrawOrigin = pallet_evm::EnsureAddressTruncated;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    let pair = ecdsa::Pair::from_seed(&ECDSA_SEED);
    let account = MultiSigner::from(pair.public()).into_account();
    let _ = pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(account, 1_000_000_000)],
    }
    .assimilate_storage(&mut storage);
    storage.into()
}

/// Simple `eth_sign` implementation, should be equal to exported by RPC
fn eth_sign(seed: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let call_msg = ethereum::signable_message(data);
    let ecdsa_msg = libsecp256k1::Message::parse(&keccak_256(&call_msg));
    let secret = libsecp256k1::SecretKey::parse(&seed).expect("valid seed");
    let (signature, recovery_id) = libsecp256k1::sign(&ecdsa_msg, &secret);
    let mut out = Vec::new();
    out.extend_from_slice(&signature.serialize()[..]);
    // Fix recovery ID: Ethereum uses 27/28 notation
    out.push(recovery_id.serialize() + 27);
    out
}

#[test]
fn eth_sign_works() {
    let seed = hex!["ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"];
    let text = b"Hello Parallel";
    let signature = [
        209, 162, 214, 50, 7, 120, 147, 185, 44, 45, 177, 156, 213, 76, 58, 169, 86, 150, 77, 186,
        5, 239, 83, 216, 102, 168, 51, 69, 179, 41, 204, 96, 108, 14, 138, 88, 253, 28, 113, 64,
        56, 76, 140, 168, 20, 62, 187, 142, 76, 200, 144, 176, 158, 2, 254, 119, 102, 5, 178, 215,
        131, 148, 248, 12, 28,
    ];
    assert_eq!(eth_sign(&seed, &text[..]), signature);
}

#[test]
fn invalid_signature() {
    let bob: <Runtime as frame_system::Config>::AccountId = Keyring::Bob.into();
    let alice: <Runtime as frame_system::Config>::AccountId = Keyring::Alice.into();
    let call = pallet_balances::Call::<Runtime>::transfer {
        dest: alice.clone(),
        value: 1_000,
    }
    .into();
    let signature = Vec::from(&hex!["dd0992d40e5cdf99db76bed162808508ac65acd7ae2fdc8573594f03ed9c939773e813181788fc02c3c68f3fdc592759b35f6354484343e18cb5317d34dab6c61b"][..]);
    new_test_ext().execute_with(|| {
        assert_err!(
            EVMSignatures::call(RuntimeOrigin::none(), Box::new(call), bob, signature, 0),
            Error::<Runtime>::InvalidSignature,
        );
    });
}

#[test]
fn balance_transfer() {
    new_test_ext().execute_with(|| {
        let pair = ecdsa::Pair::from_seed(&ECDSA_SEED);
        let account = MultiSigner::from(pair.public()).into_account();

        let alice: <Runtime as frame_system::Config>::AccountId = Keyring::Alice.into();
        assert_eq!(System::account(alice.clone()).data.free, 0);

        let call: RuntimeCall = pallet_balances::Call::<Runtime>::transfer {
            dest: alice.clone(),
            value: 1_000,
        }
        .into();
        let payload = (0xff50u16, 0u32, call.clone());
        let signature = eth_sign(&ECDSA_SEED, payload.encode().as_ref()).into();

        assert_eq!(System::account(account.clone()).nonce, 0);
        assert_ok!(EVMSignatures::call(
            RuntimeOrigin::none(),
            Box::new(call.clone()),
            account.clone(),
            signature,
            0,
        ));
        assert_eq!(System::account(alice.clone()).data.free, 1_000);
        assert_eq!(System::account(account.clone()).nonce, 1);
        assert_eq!(System::account(account.clone()).data.free, 999_998_958);

        let signature = eth_sign(&ECDSA_SEED, payload.encode().as_ref()).into();
        assert_err!(
            EVMSignatures::call(
                RuntimeOrigin::none(),
                Box::new(call.clone()),
                account.clone(),
                signature,
                0,
            ),
            Error::<Runtime>::BadNonce,
        );

        let payload = (0xff50u16, 1u32, call.clone());
        let signature = eth_sign(&ECDSA_SEED, payload.encode().as_ref()).into();
        assert_eq!(System::account(account.clone()).nonce, 1);
        assert_ok!(EVMSignatures::call(
            RuntimeOrigin::none(),
            Box::new(call.clone()),
            account.clone(),
            signature,
            1,
        ));
        assert_eq!(System::account(alice).data.free, 2_000);
        assert_eq!(System::account(account.clone()).nonce, 2);
        assert_eq!(System::account(account.clone()).data.free, 999_997_916);
    })
}

#[test]
fn call_fixtures() {
    use sp_core::crypto::Ss58Codec;

    let seed = hex!["ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"];
    let pair = ecdsa::Pair::from_seed(&seed);
    assert_eq!(
        MultiSigner::from(pair.public())
            .into_account()
            .to_ss58check(),
        "5EGynCAEvv8NLeHx8vDMvb8hTcEcMYUMWCDQEEncNEfNWB2W",
    );

    let dest =
        AccountId::from_ss58check("5GVwcV6EzxxYbXBm7H6dtxc9TCgL4oepMXtgqWYEc3VXJoaf").unwrap();
    let call: RuntimeCall = pallet_balances::Call::<Runtime>::transfer { dest, value: 1000 }.into();
    assert_eq!(
        call.encode(),
        hex!["0000c4305fb88b6ccb43d6552dc11d18e7b0ee3185247adcc6e885eb284adf6c563da10f"],
    );

    let payload = (0xff50u16, 0u32, call.clone());
    assert_eq!(
        payload.encode(),
        hex![
            "50ff000000000000c4305fb88b6ccb43d6552dc11d18e7b0ee3185247adcc6e885eb284adf6c563da10f"
        ],
    );

    let signature = hex!["6ecb474240df46ee5cde8f51cf5ccf4c75d15ac3c1772aea6c8189604263c98b16350883438c4eaa447ebcb6889d516f70351fd704bb3521072cd2fccc7c99dc1c"];
    assert_eq!(eth_sign(&seed, payload.encode().as_ref()), signature)
}
