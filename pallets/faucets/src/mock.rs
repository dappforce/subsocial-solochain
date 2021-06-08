// Creating mock runtime here
use crate::{Module, Trait, Faucet, FaucetUpdate};

use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, RuntimeDebug, Perbill, Storage
};

use frame_support::{
    impl_outer_origin, impl_outer_dispatch, parameter_types,
    assert_ok,
    weights::Weight,
    dispatch::DispatchResult,
};
use frame_system::{self as system, EnsureRoot};

impl_outer_origin! {
    pub enum Origin for Test {}
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        frame_system::System,
        pallet_balances::Balances,
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Trait for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
}

parameter_types! {
    pub const MinHandleLen: u32 = 5;
    pub const MaxHandleLen: u32 = 50;
}

impl pallet_utils::Trait for Test {
    type Event = ();
    type Currency = Balances;
    type MinHandleLen = MinHandleLen;
    type MaxHandleLen = MaxHandleLen;
    type AllowTransfers = EnsureRoot<AccountId>;
}

impl Trait for Test {
    type Event = ();
    type Currency = Balances;
}

pub(crate) type System = system::Module<Test>;
pub(crate) type Balances = pallet_balances::Module<Test>;
pub(crate) type Faucets = Module<Test>;

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u64;

pub struct ExtBuilder;

impl ExtBuilder {
    fn configure_storages(storage: &mut Storage) {
        let mut faucet_accounts = Vec::new();
        // FAUCET9 should have no balance
        for faucet in FAUCET1..=FAUCET8 {
            faucet_accounts.push(faucet);
        }

        let _ = pallet_balances::GenesisConfig::<Test> {
            balances: faucet_accounts.iter().cloned().map(|k|(k, FAUCET_INITIAL_BALANCE)).collect(),
        }.assimilate_storage(storage);
    }

    pub fn build() -> TestExternalities {
        let mut storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        Self::configure_storages(&mut storage);

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    pub fn build_with_faucet() -> TestExternalities {
        let mut storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        Self::configure_storages(&mut storage);

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);

            assert_ok!(_add_default_faucet());
        });

        ext
    }

    // TODO do we really need this func? it's much clearer to call _do_default_drip() in a few tests directly.
    pub fn build_with_one_default_drip() -> TestExternalities {
        let mut storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        Self::configure_storages(&mut storage);

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);

            assert_ok!(_add_default_faucet());

            System::set_block_number(INITIAL_BLOCK_NUMBER);
            assert_ok!(_do_default_drip());
        });

        ext
    }
}

pub(crate) const FAUCET_INITIAL_BALANCE: Balance = 400;

pub(crate) const FAUCET1: AccountId = 1;
pub(crate) const FAUCET2: AccountId = 2;
pub(crate) const FAUCET8: AccountId = 8;
pub(crate) const FAUCET9: AccountId = 9;

pub(crate) const ACCOUNT1: AccountId = 11;

pub(crate) const INITIAL_BLOCK_NUMBER: BlockNumber = 20;

pub(crate) const fn default_faucet() -> Faucet<Test> {
    Faucet {
        enabled: true,
        period: 100,
        period_limit: 50,
        drip_limit: 25,

        next_period_at: 0,
        dripped_in_current_period: 0,
    }
}

pub(crate) const fn default_faucet_update() -> FaucetUpdate<BlockNumber, Balance> {
    FaucetUpdate {
        enabled: None,
        period: Some(7_200),
        period_limit: Some(100),
        drip_limit: Some(50)
    }
}

pub(crate) fn _add_default_faucet() -> DispatchResult {
    _add_faucet(None, None)
}

pub(crate) fn _add_faucet(
    origin: Option<Origin>,
    faucet_account: Option<AccountId>,
) -> DispatchResult {
    let settings =  default_faucet();
    Faucets::add_faucet(
        origin.unwrap_or_else(Origin::root),
        faucet_account.unwrap_or(FAUCET1),
        settings.period,
        settings.period_limit,
        settings.drip_limit
    )
}

pub(crate) fn _update_default_faucet() -> DispatchResult {
    _update_faucet(None, None, None)
}

pub(crate) fn _update_faucet_settings(settings: FaucetUpdate<BlockNumber, Balance>) -> DispatchResult {
    _update_faucet(None, None, Some(settings))
}

pub(crate) fn _update_faucet(
    origin: Option<Origin>,
    faucet_account: Option<AccountId>,
    update: Option<FaucetUpdate<BlockNumber, Balance>>
) -> DispatchResult {
    Faucets::update_faucet(
        origin.unwrap_or_else(Origin::root),
        faucet_account.unwrap_or(FAUCET1),
        update.unwrap_or_else(default_faucet_update),
    )
}

pub(crate) fn _remove_default_faucet() -> DispatchResult {
    _remove_faucets(None, None)
}

pub(crate) fn _remove_faucets(
    origin: Option<Origin>,
    faucet_accounts: Option<Vec<AccountId>>,
) -> DispatchResult {
    Faucets::remove_faucets(
        origin.unwrap_or_else(Origin::root),
        faucet_accounts.unwrap_or_else(|| vec![FAUCET1])
    )
}

pub(crate) fn _do_default_drip() -> DispatchResult {
    _drip(None, None, None)
}

pub(crate) fn _drip(
    origin: Option<Origin>,
    recipient: Option<AccountId>,
    amount: Option<Balance>
) -> DispatchResult {
    Faucets::drip(
        origin.unwrap_or_else(|| Origin::signed(FAUCET1)),
        recipient.unwrap_or(ACCOUNT1),
        amount.unwrap_or(default_faucet().drip_limit)
    )
}
