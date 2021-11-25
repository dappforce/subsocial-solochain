use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Storage
};

use crate as dotsama_claims;

use frame_support::{
    parameter_types,
    assert_ok,
    dispatch::DispatchResultWithPostInfo,
    traits::Everything,
};
use frame_system as system;

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
        Utils: pallet_utils::{Pallet, Event<T>},
        DotsamaClaims: dotsama_claims::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 28;
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

impl pallet_utils::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MinHandleLen = ();
    type MaxHandleLen = ();
}

parameter_types! {
    pub const InitialClaimAmount: Balance = 10;
    pub const AccountsSetLimit: u32 = 100;
}

impl dotsama_claims::Config for Test {
    type Event = Event;
    type InitialClaimAmount = InitialClaimAmount;
    type AccountsSetLimit = AccountsSetLimit;
    type WeightInfo = ();
}

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u64;

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;

pub(crate) const REWARDS_SENDER: AccountId = 10;
pub(crate) const ALT_REWARDS_SENDER: AccountId = 11;

pub struct ExtBuilder;

impl ExtBuilder {
    fn configure_balances_of_rewards_senders(balance: Balance) -> impl Fn(&mut Storage) {
        move |storage: &mut Storage| {
            let _ = pallet_balances::GenesisConfig::<Test> {
                balances: [REWARDS_SENDER, ALT_REWARDS_SENDER].iter().cloned().map(|acc| (acc, balance)).collect(),
            }.assimilate_storage(storage);
        }
    }

    fn build_with_custom_balances_for_rewards_senders(balance: Balance) -> TestExternalities {
        let storage = &mut system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        Self::configure_balances_of_rewards_senders(balance)(storage);

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    pub(crate) fn build() -> TestExternalities {
        let sufficient_balance = ExistentialDeposit::get() + InitialClaimAmount::get();
        Self::build_with_custom_balances_for_rewards_senders(sufficient_balance)
    }

    pub(crate) fn build_with_insufficient_balances_for_rewards_sender() -> TestExternalities {
        let insufficient_balance = ExistentialDeposit::get() + InitialClaimAmount::get() - 1;
        Self::build_with_custom_balances_for_rewards_senders(insufficient_balance)
    }

    pub(crate) fn build_with_set_rewards_sender() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(|| {
            assert_ok!(_set_rewards_sender(None, Some(REWARDS_SENDER).into()));
        });

        ext
    }

    pub(crate) fn build_with_set_rewards_sender_and_eligible_accounts() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(|| {
            // Set a rewards sender
            assert_ok!(_set_rewards_sender(None, Some(REWARDS_SENDER).into()));

            // Add eligible accounts
            let eligible_accounts = vec![ACCOUNT1, ACCOUNT2];
            assert_ok!(DotsamaClaims::add_eligible_accounts(Origin::root(), eligible_accounts));
        });

        ext
    }
}

pub(crate) fn _claim_tokens_by_account1() -> DispatchResultWithPostInfo {
    _claim_tokens(None)
}

pub(crate) fn _claim_tokens_by_account2() -> DispatchResultWithPostInfo {
    _claim_tokens(Some(Origin::signed(ACCOUNT2)))
}

/// If no origin specified, tokens will be claimed by account 1.
pub(crate) fn _claim_tokens(origin: Option<Origin>) -> DispatchResultWithPostInfo {
    DotsamaClaims::claim_tokens(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
    )
}

pub(crate) fn _set_default_rewards_sender() -> DispatchResultWithPostInfo {
    _set_rewards_sender(None, None)
}

pub(crate) fn _not_root_tries_to_set_rewards_sender() -> DispatchResultWithPostInfo {
    _set_rewards_sender(Some(Origin::signed(ACCOUNT1)), None)
}

pub(crate) fn _set_rewards_sender(
    origin: Option<Origin>,
    account: Option<Option<AccountId>>,
) -> DispatchResultWithPostInfo {
    DotsamaClaims::set_rewards_sender(
        origin.unwrap_or_else(Origin::root),
        account.unwrap_or(Some(REWARDS_SENDER)),
    )
}

pub(crate) fn _add_eligible_accounts_over_limit() -> DispatchResultWithPostInfo {
    _add_eligible_accounts(None, vec![ACCOUNT1; AccountsSetLimit::get() as usize + 1])
}

pub(crate) fn _not_root_tries_to_add_eligible_accounts() -> DispatchResultWithPostInfo {
    _add_eligible_accounts(Some(Origin::signed(ACCOUNT1)), vec![ACCOUNT1])
}

pub(crate) fn _add_eligible_accounts(
    origin: Option<Origin>,
    eligible_accounts: Vec<AccountId>
) -> DispatchResultWithPostInfo {
    DotsamaClaims::add_eligible_accounts(
        origin.unwrap_or_else(Origin::root),
        eligible_accounts
    )
}
