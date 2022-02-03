use std::borrow::BorrowMut;
use std::cell::RefCell;
use frame_benchmarking::whitelisted_caller;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Storage
};

use crate as pallet_locker_mirror;

use frame_support::{ord_parameter_types, parameter_types};
use frame_support::traits::{EnsureOrigin, SortedMembers};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSignedBy, RawOrigin};


pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;


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
        LockerMirror: pallet_locker_mirror::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 28;
}

impl system::Config for Test {
    type BaseCallFilter = ();
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

thread_local! {
    pub static TEST_ORACLE_ORIGIN: RefCell<AccountId> = RefCell::new(whitelisted_caller());
}
pub struct OracleOriginSortedMembers;
impl SortedMembers<AccountId> for OracleOriginSortedMembers {
    fn sorted_members() -> Vec<AccountId> {
        vec![TEST_ORACLE_ORIGIN.with(|v| v.borrow().clone())]
    }
}
impl pallet_locker_mirror::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type OracleOrigin = EnsureSignedBy<OracleOriginSortedMembers, AccountId>;
    type WeightInfo = ();
}


pub struct ExtBuilder {
    oracle_origin: AccountId,
}
impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            oracle_origin: whitelisted_caller(),
        }
    }
}
impl ExtBuilder {
    pub fn oracle_account_id(mut self, oracle_origin: AccountId) -> Self {
        self.oracle_origin = oracle_origin;
        self
    }

    pub fn set_associated_consts(&self) {
        TEST_ORACLE_ORIGIN.with(|v| *v.borrow_mut() = self.oracle_origin);
    }

    pub fn build(self) -> TestExternalities {
        self.set_associated_consts();

        let storage = &mut system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| System::set_block_number(1));

        ext
    }
}