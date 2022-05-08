use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header
};

use crate as faucets;

use frame_support::{
    parameter_types,
    traits::Everything,
};
use frame_system as system;
use pallet_utils::{DEFAULT_MIN_HANDLE_LEN, DEFAULT_MAX_HANDLE_LEN};

use super::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Faucets: faucets::{Pallet, Call, Storage, Event<T>},
        Utils: pallet_utils::{Pallet, Storage, Event<T>, Config<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}
impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
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
    type Balance = u64;
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

impl Config for Test {
    type Event = Event;
    type Currency = Balances;
}

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u64;

pub struct ExtBuilder;

impl ExtBuilder {

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
        }
        .assimilate_storage(&mut t)
            .unwrap();
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    // TODO: Add a default build with faucet

}

pub(crate) const fn default_faucet_update(
    enabled: Option<bool>, 
    period: Option<BlockNumber>, 
    period_limit: Option<Balance>, 
    drip_limit: Option<Balance>
) -> FaucetUpdate<BlockNumber, Balance> {
    FaucetUpdate {
        enabled,
        period,
        period_limit,
        drip_limit
    }
}
