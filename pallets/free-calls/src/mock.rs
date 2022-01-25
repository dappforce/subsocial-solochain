use crate as pallet_free_calls;
use frame_support::{parameter_types, sp_io};
use frame_system::EnsureRoot;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Storage
};
pub use subsocial_primitives::{AccountId, Signature, Balance, Index};
use subsocial_primitives::BlockNumber;
use crate::mock::time::*;
use crate::WindowConfig;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;


pub mod time {
    use subsocial_primitives::{Moment, BlockNumber};

    pub const MILLISECS_PER_BLOCK: Moment = 6000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		FreeCalls: pallet_free_calls::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

impl pallet_free_calls::Config for Test {
    type Event = Event;
    type Call = Call;
    const WINDOWS_CONFIG: &'static [WindowConfig<Self::BlockNumber>] = &[
        WindowConfig::new(1 * DAYS, 1),
        WindowConfig::new(2 * HOURS, 3),
        WindowConfig::new(30 * MINUTES, 5),
        WindowConfig::new(5 * MINUTES, 20),
        WindowConfig::new(1, 1000),
    ];
    type ManagerOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}