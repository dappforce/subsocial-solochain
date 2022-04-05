use std::cell::RefCell;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{generic, traits::{BlakeTwo256, IdentityLookup}};

pub use crate as pallet_free_calls;

use crate::test_pallet;

use frame_support::{
    parameter_types,
};
use frame_support::traits::{Contains};
use frame_system as system;
use frame_system::{EnsureRoot};
use pallet_locker_mirror::{BalanceOf, LockedInfoOf};
use crate::config::{ConfigHash, RateLimiterConfig, WindowConfig};
use crate::max_quota_percentage;
use crate::quota::NumberOfCalls;

pub(crate) type AccountId = subsocial_primitives::AccountId;
pub(crate) type BlockNumber = subsocial_primitives::BlockNumber;
pub(crate) type Balance = subsocial_primitives::Balance;
/// Opaque block header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Opaque block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Opaque block identifier type.
pub type BlockId = generic::BlockId<Block>;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: system::{Pallet, Call, Config, Storage, Event<T>},
        FreeCalls: pallet_free_calls::{Pallet, Call, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        LockerMirror: pallet_locker_mirror::{Pallet, Call, Storage, Event<T>},
        TestPallet: test_pallet::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const SS58Prefix: u8 = 28;
}

pub struct TestBaseCallFilter;
impl Contains<Call> for TestBaseCallFilter {
    fn contains(c: &Call) -> bool {
        match *c {
            Call::FreeCalls(_) => true,
            // For benchmarking, this acts as a noop call
            Call::System(frame_system::Call::remark { .. }) => true,
            // For tests
            Call::TestPallet(_) => true,
            _ => false,
        }
    }
}

impl system::Config for Test {
    type BaseCallFilter = TestBaseCallFilter;
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
    type AccountData = pallet_balances::AccountData<Balance>;
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


impl pallet_locker_mirror::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type OracleOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}

impl test_pallet::Config for Test {
    type Event = Event;
}

////// Free Call Dependencies


type CallFilterFn = fn(&Call) -> bool;
static DEFAULT_CALL_FILTER_FN: CallFilterFn = |_| true;

type QuotaCalculationFn<T> = fn(<T as frame_system::Config>::BlockNumber, Option<LockedInfoOf<T>>) -> Option<NumberOfCalls>;
static DEFAULT_QUOTA_CALCULATION_FN: QuotaCalculationFn<Test> = |current_block, locked_info| {
    return Some(10);
};

pub static DEFAULT_CONFIG_HASH: ConfigHash = 0;

pub static DEFAULT_WINDOWS_CONFIG: [WindowConfig<BlockNumber>; 1] = [
    WindowConfig::new(10, max_quota_percentage!(100)),
];

parameter_types! {
    pub static TestRateLimiterConfig: RateLimiterConfig<BlockNumber> = RateLimiterConfig::new(
        DEFAULT_WINDOWS_CONFIG.to_vec(),
        DEFAULT_CONFIG_HASH,
    );
    pub const AccountsSetLimit: u32 = 10;
}

thread_local! {
    pub static CALL_FILTER: RefCell<CallFilterFn> = RefCell::new(DEFAULT_CALL_FILTER_FN);
    pub static QUOTA_CALCULATION: RefCell<QuotaCalculationFn<Test>> = RefCell::new(DEFAULT_QUOTA_CALCULATION_FN);
}

pub struct TestCallFilter;
impl Contains<Call> for TestCallFilter {
    fn contains(call: &Call) -> bool {
        CALL_FILTER.with(|filter| filter.borrow()(call))
    }
}

pub struct TestQuotaCalculation;
impl pallet_free_calls::quota_strategy::MaxQuotaCalculationStrategy<<Test as frame_system::Config>::BlockNumber, BalanceOf<Test>> for TestQuotaCalculation {
    fn calculate(
        current_block: <Test as frame_system::Config>::BlockNumber,
        locked_info: Option<LockedInfoOf<Test>>
    ) -> Option<NumberOfCalls> {
        QUOTA_CALCULATION.with(|strategy| strategy.borrow()(current_block, locked_info))
    }
}

impl pallet_free_calls::Config for Test {
    type Event = Event;
    type Call = Call;
    type RateLimiterConfig = TestRateLimiterConfig;
    type CallFilter = TestCallFilter;
    type WeightInfo = ();
    type MaxQuotaCalculationStrategy = TestQuotaCalculation;
    type AccountsSetLimit = AccountsSetLimit;
}

pub struct ExtBuilder {
    call_filter: CallFilterFn,
    quota_calculation: QuotaCalculationFn<Test>,
    windows_config: Vec<WindowConfig<BlockNumber>>,
    config_hash: ConfigHash,
}
impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            call_filter: DEFAULT_CALL_FILTER_FN,
            quota_calculation: DEFAULT_QUOTA_CALCULATION_FN,
            windows_config: DEFAULT_WINDOWS_CONFIG.to_vec(),
            config_hash: DEFAULT_CONFIG_HASH,
        }
    }
}
impl ExtBuilder {
    pub fn call_filter(mut self, call_filter: CallFilterFn) -> Self {
        self.call_filter = call_filter;
        self
    }

    pub fn quota_calculation(mut self, quota_calculation: QuotaCalculationFn<Test>) -> Self {
        self.quota_calculation = quota_calculation;
        self
    }

    pub fn windows_config(mut self, windows_config: Vec<WindowConfig<BlockNumber>>) -> Self {
        self.windows_config = windows_config;
        self
    }

    pub fn config_hash(mut self, config_hash: ConfigHash) -> Self {
        self.config_hash = config_hash;
        self
    }

    pub fn set_configs(&self) {
        CALL_FILTER.with(|filter| *filter.borrow_mut() = self.call_filter);
        QUOTA_CALCULATION.with(|calc| *calc.borrow_mut() = self.quota_calculation);
        TEST_RATE_LIMITER_CONFIG.with(|configs| *configs.borrow_mut() = RateLimiterConfig::new(self.windows_config.clone(), self.config_hash));
    }

    pub fn build(self) -> TestExternalities {
        self.set_configs();

        let storage = &mut system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| <frame_system::Pallet<Test>>::set_block_number(1));

        ext
    }
}