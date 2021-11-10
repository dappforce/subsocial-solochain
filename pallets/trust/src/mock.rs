use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	testing::Header,
};

use crate as trust;

use frame_support::{dispatch::DispatchResultWithPostInfo, parameter_types};
use frame_system as system;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		Utils: pallet_utils::{Module, Event<T>},
		Trust: trust::{Module, Call, Storage, Event<T>},
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
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
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
}

impl pallet_utils::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MinHandleLen = ();
	type MaxHandleLen = ();
}

impl trust::Config for Test {
	type Event = Event;
	type SetTrustLevel = system::EnsureRoot<AccountId>;
	type WeightInfo = ();
}

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;

pub(crate) const BAD_ORIGIN: AccountId = 10;

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build() -> sp_io::TestExternalities {
		let storage = system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| System::set_block_number(1));

		ext
	}
}

pub(crate) fn _set_email_verified_by_account1() -> DispatchResultWithPostInfo {
	_set_email_verified(None, None)
}

pub(crate) fn _set_email_verified_by_account1_bad_origin() -> DispatchResultWithPostInfo {
	_set_email_verified(Some(Origin::signed(BAD_ORIGIN)), None)
}

pub(crate) fn _set_email_verified_by_account2() -> DispatchResultWithPostInfo {
	_set_email_verified(None, Some(ACCOUNT2))
}

pub(crate) fn _set_phone_number_verified_by_account1() -> DispatchResultWithPostInfo {
	_set_phone_number_verified(None, None)
}

pub(crate) fn _set_phone_number_verified_by_account1_bad_origin() -> DispatchResultWithPostInfo {
	_set_phone_number_verified(Some(Origin::signed(BAD_ORIGIN)), None)
}

pub(crate) fn _set_phone_number_verified_by_account2() -> DispatchResultWithPostInfo {
	_set_phone_number_verified(None, Some(ACCOUNT2))
}

pub(crate) fn _set_email_verified(
	origin: Option<Origin>,
	who: Option<AccountId>,
) -> DispatchResultWithPostInfo {
	Trust::set_email_verified(
		origin.unwrap_or_else(|| Origin::root()),
		who.unwrap_or(ACCOUNT1),
	)
}

pub(crate) fn _set_phone_number_verified(
	origin: Option<Origin>,
	who: Option<AccountId>,
) -> DispatchResultWithPostInfo {
	Trust::set_phone_number_verified(
		origin.unwrap_or_else(|| Origin::root()),
		who.unwrap_or(ACCOUNT1),
	)
}
