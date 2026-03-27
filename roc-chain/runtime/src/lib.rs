//! BlocRoc Runtime — FRAME-based WASM runtime for the BlocRoc blockchain.
//!
//! This runtime wires together all BlocRoc pallets alongside standard Substrate
//! system pallets. It is compiled to WASM and embedded in the node binary.

#![cfg_attr(not(feature = "std"), no_std)]
// Allow the runtime to use `construct_runtime!` which generates clippy warnings
#![allow(clippy::identity_op)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{Decode, Encode};
use frame_support::{
    construct_runtime, derive_impl,
    parameter_types,
    traits::{ConstBool, ConstU128, ConstU32, ConstU64, ConstU8},
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        IdentityFee, Weight,
    },
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One, Verify},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, MultiSignature,
};
use sp_genesis_builder::{self, PresetId};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// BlocRoc pallet imports
pub use pallet_event;
pub use pallet_marketplace;
pub use pallet_scanner;
pub use pallet_ticket;

// ─── Primitive types ─────────────────────────────────────────────────────────

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type Nonce = u32;
pub type Hash = sp_core::H256;

pub mod opaque {
    use super::*;
    use sp_runtime::generic;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    pub type BlockId = generic::BlockId<Block>;
    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

// ─── Runtime version ─────────────────────────────────────────────────────────

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("roc-chain"),
    impl_name: create_runtime_str!("roc-chain"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

// ─── Block / weight limits ────────────────────────────────────────────────────

const NORMAL_DISPATCH_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(75);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    pub BlockWeightsConfig: BlockWeights = BlockWeights::with_sensible_defaults(
        Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
        NORMAL_DISPATCH_RATIO,
    );
    pub BlockLengthConfig: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
}

// ─── System pallet ────────────────────────────────────────────────────────────

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = BlockWeightsConfig;
    type BlockLength = BlockLengthConfig;
    type AccountId = AccountId;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Block = Block;
    type BlockHashCount = BlockHashCount;
    type DbWeight = RocksDbWeight;
    type Version = Version;
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

// ─── Consensus pallets ────────────────────────────────────────────────────────

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 500; // 500ms slot duration
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// ─── Balances ─────────────────────────────────────────────────────────────────

parameter_types! {
    pub const ExistentialDeposit: Balance = 500;
}

impl pallet_balances::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = RuntimeHoldReason;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type MaxFreezes = ConstU32<1>;
    type RuntimeFreezeReason = RuntimeFreezeReason;
}

// ─── Transaction payment ──────────────────────────────────────────────────────

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

// ─── Sudo ─────────────────────────────────────────────────────────────────────

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

// ─── BlocRoc pallets ──────────────────────────────────────────────────────────

impl pallet_ticket::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxStringLength = ConstU32<128>;
    type WeightInfo = pallet_ticket::weights::SubstrateWeight<Runtime>;
}

impl pallet_event::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_event::weights::SubstrateWeight<Runtime>;
}

impl pallet_marketplace::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_marketplace::weights::SubstrateWeight<Runtime>;
}

impl pallet_scanner::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_scanner::weights::SubstrateWeight<Runtime>;
}

// ─── construct_runtime! ───────────────────────────────────────────────────────

construct_runtime!(
    pub enum Runtime {
        // System
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,

        // Consensus
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,

        // BlocRoc
        Ticket: pallet_ticket,
        Event: pallet_event,
        Marketplace: pallet_marketplace,
        Scanner: pallet_scanner,
    }
);

// ─── Runtime aliases ─────────────────────────────────────────────────────────

pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type SignedBlock = generic::SignedBlock<Block>;
pub type BlockId = generic::BlockId<Block>;
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, SignedExtra>;
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

// ─── Runtime API implementations ─────────────────────────────────────────────

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion { VERSION }
        fn execute_block(block: Block) { Executive::execute_block(block); }
        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata { OpaqueMetadata::new(Runtime::metadata().into()) }
        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }
        fn metadata_versions() -> Vec<u32> { Runtime::metadata_versions() }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }
        fn finalize_block() -> <Block as BlockT>::Header { Executive::finalize_block() }
        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }
        fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(source: TransactionSource, tx: <Block as BlockT>::Extrinsic, block_hash: <Block as BlockT>::Hash) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }
        fn authorities() -> Vec<AuraId> { Aura::authorities().into_inner() }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }
        fn decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }
        fn current_set_id() -> sp_consensus_grandpa::SetId { Grandpa::current_set_id() }
        fn submit_report_equivocation_unsigned_extrinsic(_: sp_consensus_grandpa::EquivocationProof<<Block as BlockT>::Hash, NumberFor<Block>>, _: sp_consensus_grandpa::OpaqueKeyOwnershipProof) -> Option<()> { None }
        fn generate_key_ownership_proof(_: sp_consensus_grandpa::SetId, _: GrandpaId) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> { None }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce { System::account_nonce(account) }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance { TransactionPayment::weight_to_fee(weight) }
        fn query_length_to_fee(length: u32) -> Balance { TransactionPayment::length_to_fee(length) }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            sp_genesis_builder::build_state::<RuntimeGenesisConfig>(config)
        }
        fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
            sp_genesis_builder::get_preset::<RuntimeGenesisConfig>(id, |_| None)
        }
        fn preset_names() -> Vec<PresetId> { vec![] }
    }
}
