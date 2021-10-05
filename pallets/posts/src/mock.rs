use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use pallet_permissions::default_permissions::DefaultSpacePermissions;

use crate as posts;

use frame_support::parameter_types;
use frame_system as system;

use pallet_utils::{DEFAULT_MIN_HANDLE_LEN, DEFAULT_MAX_HANDLE_LEN};

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
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Permissions: pallet_permissions::{Module, Call},
        Profiles: pallet_profiles::{Module, Call, Storage, Event<T>},
        Spaces: pallet_spaces::{Module, Call, Storage, Event<T>},
        SpaceFollows: pallet_space_follows::{Module, Call, Storage, Event<T>},
        Utils: pallet_utils::{Module, Event<T>},
        Posts: posts::{Module, Call, Storage, Event<T>},
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

impl pallet_permissions::Config for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl pallet_profiles::Config for Test {
    type Event = Event;
    type AfterProfileUpdated = ();
    type WeightInfo = ();
}

impl pallet_space_follows::Config for Test {
    type Event = Event;
    type BeforeSpaceFollowed = ();
    type BeforeSpaceUnfollowed = ();
    type WeightInfo = ();
}

impl pallet_spaces::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type Roles = ();
    type SpaceFollows = ();
    type BeforeSpaceCreated = ();
    type AfterSpaceUpdated = ();
    type IsAccountBlocked = ();
    type IsContentBlocked = ();
    type HandleDeposit = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxCommentDepth: u32 = 5;
}

impl posts::Config for Test {
    type Event = Event;
    type MaxCommentDepth = MaxCommentDepth;
    type PostScores = ();
    type AfterPostUpdated = ();
    type IsPostBlocked = ();
    type WeightInfo = ();
}

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;
