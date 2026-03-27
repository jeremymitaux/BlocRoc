use crate as pallet_ticket;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Ticket: pallet_ticket,
    }
);

// ── frame_system ──────────────────────────────────────────────────────────────

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    // Must match pallet_balances::AccountData so Balances can store account info.
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = RuntimeTask;
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

// ── pallet_balances ───────────────────────────────────────────────────────────

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ConstU32<10>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = RuntimeHoldReason;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<1>;
    type RuntimeFreezeReason = RuntimeFreezeReason;
}

// ── pallet_ticket ─────────────────────────────────────────────────────────────

parameter_types! {
    /// Maximum byte length for all `BoundedVec<u8, _>` string fields.
    pub const MaxStringLength: u32 = 64;
}

impl pallet_ticket::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxStringLength = MaxStringLength;
    type WeightInfo = ();
}

// ── Test externalities ────────────────────────────────────────────────────────

/// Accounts used in tests — pre-funded with enough balance for all scenarios.
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const SCANNER: u64 = 4;

pub const INITIAL_BALANCE: u64 = 1_000_000;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    // Pre-fund test accounts so purchase/transfer tests work without
    // separately setting up balances every time.
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, INITIAL_BALANCE),
            (BOB, INITIAL_BALANCE),
            (CHARLIE, INITIAL_BALANCE),
            (SCANNER, INITIAL_BALANCE),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(storage);
    // Start at block 1 so events are deposited (block 0 has no event storage).
    ext.execute_with(|| frame_system::Pallet::<Test>::set_block_number(1));
    ext
}
