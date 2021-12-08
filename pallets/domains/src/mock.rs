use frame_benchmarking::frame_support::dispatch::DispatchResult;
use crate as pallet_domains;
use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use df_traits::SpacesProvider;

use pallet_utils::{DEFAULT_MAX_HANDLE_LEN, DEFAULT_MIN_HANDLE_LEN, SpaceId};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Storage},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Utils: pallet_utils::{Pallet, Config<T>, Event<T>},
		Domains: pallet_domains::{Pallet, Call, Storage, Event<T>},
	}
);

type Balance = u64;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

parameter_types! {
    pub const MinHandleLen: u32 = DEFAULT_MIN_HANDLE_LEN;
    pub const MaxHandleLen: u32 = DEFAULT_MAX_HANDLE_LEN;
}

impl pallet_utils::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MinHandleLen = MinHandleLen;
    type MaxHandleLen = MaxHandleLen;
}

parameter_types! {
    pub const MinTldLength: u8 = 2;
    pub const MinDomainLength: u8 = 3;
    pub const MaxDomainLength: u8 = 63;

    pub const ReservationPeriodLimit: u32 = 100;
    pub const OuterValueLimit: u16 = 256;
    pub const OuterValueDeposit: Balance = 1;

    pub const DomainsInsertLimit: u32 = 30_000;
}

impl pallet_domains::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type SpacesProvider = Self;
    type MinTldLength = MinTldLength;
    type MinDomainLength = MinDomainLength;
    type MaxDomainLength = MaxDomainLength;
    type ReservationPeriodLimit = ReservationPeriodLimit;
    type OuterValueLimit = OuterValueLimit;
    type OuterValueDepositPerByte = OuterValueDeposit;
    type DomainsInsertLimit = DomainsInsertLimit;
    type WeightInfo = ();
}

pub(crate) const EXISTING_SPACE: u64 = 1;
// pub(crate) const NON_EXISTING_SPACE: u64 = 2;

impl SpacesProvider for Test {
    fn ensure_space_exists(space_id: SpaceId) -> DispatchResult {
        match space_id == EXISTING_SPACE {
            true => Ok(()),
            false => Err("SpaceNotFound".into()),
        }
    }
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
