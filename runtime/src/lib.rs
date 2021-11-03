//! The Subsocial Node runtime.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_std::{
	prelude::*,
	collections::btree_map::BTreeMap,
};
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
pub use subsocial_primitives::{AccountId, Signature, Balance, Index};
use subsocial_primitives::{BlockNumber, Hash, Moment};
use sp_runtime::{
    ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys,
    transaction_validity::{TransactionValidity, TransactionSource},
};
use sp_runtime::traits::{
    BlakeTwo256, Block as BlockT, NumberFor, AccountIdLookup
};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_balances::Call as BalancesCall;
pub use sp_runtime::{Permill, Perbill};
pub use frame_support::{
    construct_runtime, parameter_types, StorageValue,
    traits::{KeyOwnerProofSystem, Randomness, Currency, Imbalance, OnUnbalanced, Filter},
    weights::{
        Weight, IdentityFee, DispatchClass,
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
    },
};
use frame_system::{
    EnsureRoot,
    limits::{BlockWeights, BlockLength}
};
use pallet_transaction_payment::CurrencyAdapter;
use static_assertions::const_assert;

use pallet_permissions::SpacePermission;
use pallet_posts::rpc::{FlatPost, FlatPostKind, RepliesByPostId};
use pallet_profiles::rpc::FlatSocialAccount;
use pallet_reactions::{
	ReactionId,
	ReactionKind,
	rpc::FlatReaction,
};
use pallet_spaces::rpc::FlatSpace;
use pallet_utils::{SpaceId, PostId, DEFAULT_MIN_HANDLE_LEN, DEFAULT_MAX_HANDLE_LEN};

pub mod constants;
use constants::{currency::*, time::*};

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("subsocial"),
	impl_name: create_runtime_str!("dappforce-subsocial"),
	authoring_version: 0,
	spec_version: 14,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 2,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item=NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            let mut fees_with_maybe_tips = fees;
            fees_with_maybe_tips.maybe_subsume(fees_then_tips.next());
            Utils::on_unbalanced(fees_with_maybe_tips);
        }
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 28;
}

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = BaseFilter;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.

    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
}

impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProof =
    <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type KeyOwnerProofSystem = ();

    type HandleEquivocation = ();

    type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = Moment;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 10 * CENTS;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    /// The type for recording an account's balance.
    type Balance = Balance;
    type DustRemoval = ();
    /// The ubiquitous event type.
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = ();
}

// Subsocial custom pallets go below:
// ------------------------------------------------------------------------------------------------

parameter_types! {
  pub const MinHandleLen: u32 = DEFAULT_MIN_HANDLE_LEN;
  pub const MaxHandleLen: u32 = DEFAULT_MAX_HANDLE_LEN;
}

impl pallet_utils::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MinHandleLen = MinHandleLen;
	type MaxHandleLen = MaxHandleLen;
}

use pallet_permissions::default_permissions::DefaultSpacePermissions;

impl pallet_permissions::Config for Runtime {
	type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl tipping_lottery_pallet::Config for Runtime {
	type Currency = Balances;
	type Event = Event;
}

parameter_types! {
  pub const MaxCommentDepth: u32 = 10;
}

impl pallet_posts::Config for Runtime {
	type Event = Event;
	type MaxCommentDepth = MaxCommentDepth;
	type PostScores = Scores;
	type AfterPostUpdated = PostHistory;
	type IsPostBlocked = ()/*Moderation*/;
}

parameter_types! {}

impl pallet_post_history::Config for Runtime {}

parameter_types! {}

impl pallet_profile_follows::Config for Runtime {
	type Event = Event;
	type BeforeAccountFollowed = Scores;
	type BeforeAccountUnfollowed = Scores;
}

parameter_types! {}

impl pallet_profiles::Config for Runtime {
	type Event = Event;
	type AfterProfileUpdated = ProfileHistory;
}

parameter_types! {}

impl pallet_profile_history::Config for Runtime {}

parameter_types! {}

impl pallet_reactions::Config for Runtime {
	type Event = Event;
	type PostReactionScores = Scores;
}

parameter_types! {
  pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
}

impl pallet_roles::Config for Runtime {
	type Event = Event;
	type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
	type Spaces = Spaces;
	type SpaceFollows = SpaceFollows;
	type IsAccountBlocked = ()/*Moderation*/;
	type IsContentBlocked = ()/*Moderation*/;
}

parameter_types! {
  pub const FollowSpaceActionWeight: i16 = 7;
  pub const FollowAccountActionWeight: i16 = 3;

  pub const SharePostActionWeight: i16 = 7;
  pub const UpvotePostActionWeight: i16 = 5;
  pub const DownvotePostActionWeight: i16 = -3;

  pub const CreateCommentActionWeight: i16 = 5;
  pub const ShareCommentActionWeight: i16 = 5;
  pub const UpvoteCommentActionWeight: i16 = 4;
  pub const DownvoteCommentActionWeight: i16 = -2;
}

impl pallet_scores::Config for Runtime {
	type Event = Event;

	type FollowSpaceActionWeight = FollowSpaceActionWeight;
	type FollowAccountActionWeight = FollowAccountActionWeight;

	type SharePostActionWeight = SharePostActionWeight;
	type UpvotePostActionWeight = UpvotePostActionWeight;
	type DownvotePostActionWeight = DownvotePostActionWeight;

	type CreateCommentActionWeight = CreateCommentActionWeight;
	type ShareCommentActionWeight = ShareCommentActionWeight;
	type UpvoteCommentActionWeight = UpvoteCommentActionWeight;
	type DownvoteCommentActionWeight = DownvoteCommentActionWeight;
}

parameter_types! {}

impl pallet_space_follows::Config for Runtime {
	type Event = Event;
	type BeforeSpaceFollowed = Scores;
	type BeforeSpaceUnfollowed = Scores;
}

parameter_types! {}

impl pallet_space_ownership::Config for Runtime {
	type Event = Event;
}

parameter_types! {
	pub HandleDeposit: Balance = 5 * DOLLARS;
}

impl pallet_spaces::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type Roles = Roles;
	type SpaceFollows = SpaceFollows;
	type BeforeSpaceCreated = SpaceFollows;
	type AfterSpaceUpdated = SpaceHistory;
	type IsAccountBlocked = ()/*Moderation*/;
	type IsContentBlocked = ()/*Moderation*/;
	type HandleDeposit = HandleDeposit;
}

parameter_types! {
    pub InitialClaimAmount: Balance = 10 * DOLLARS;
    pub AccountsSetLimit: u32 = 30_000;
}

impl pallet_dotsama_claims::Config for Runtime {
    type Event = Event;
    type InitialClaimAmount = InitialClaimAmount;
    type AccountsSetLimit = AccountsSetLimit;
    type WeightInfo = pallet_dotsama_claims::weights::SubstrateWeight<Runtime>;
}

parameter_types! {}

impl pallet_space_history::Config for Runtime {}

pub struct BaseFilter;
impl Filter<Call> for BaseFilter {
    fn filter(c: &Call) -> bool {
        let is_set_balance = matches!(c, Call::Balances(pallet_balances::Call::set_balance(..)));
        let is_force_transfer = matches!(c, Call::Balances(pallet_balances::Call::force_transfer(..)));
        match *c {
            Call::Balances(..) => is_set_balance || is_force_transfer,
            _ => true,
        }
    }
}

/*
parameter_types! {
	pub const MaxSessionKeysPerAccount: u16 = 10;
	pub const BaseSessionKeyBond: Balance = 1 * DOLLARS;
}

pub struct SessionKeysProxyFilter;
impl Default for SessionKeysProxyFilter { fn default() -> Self { Self } }
impl Filter<Call> for SessionKeysProxyFilter {
	fn filter(c: &Call) -> bool {
		match *c {
			Call::SpaceFollows(..) => true,
			Call::ProfileFollows(..) => true,
			Call::Posts(..) => true,
			Call::Reactions(..) => true,
			_ => false,
		}
	}
}

impl pallet_session_keys::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type MaxSessionKeysPerAccount = MaxSessionKeysPerAccount;
	type BaseFilter = SessionKeysProxyFilter;
	type BaseSessionKeyBond = BaseSessionKeyBond;
}

impl pallet_donations::Config for Runtime {
	type Event = Event;
}

parameter_types! {
	pub const DefaultAutoblockThreshold: u16 = 20;
}

impl pallet_moderation::Config for Runtime {
	type Event = Event;
	type DefaultAutoblockThreshold = DefaultAutoblockThreshold;
}

parameter_types! {
	pub const DailyPeriodInBlocks: BlockNumber = DAYS;
	pub const WeeklyPeriodInBlocks: BlockNumber = DAYS * 7;
	pub const MonthlyPeriodInBlocks: BlockNumber = DAYS * 30;
	pub const QuarterlyPeriodInBlocks: BlockNumber = DAYS * 30 * 3;
	pub const YearlyPeriodInBlocks: BlockNumber = DAYS * 365;
}

impl pallet_subscriptions::Config for Runtime {
	type Event = Event;
	type Subscription = Call;
	type Scheduler = Scheduler;
	type DailyPeriodInBlocks = DailyPeriodInBlocks;
	type WeeklyPeriodInBlocks = WeeklyPeriodInBlocks;
	type MonthlyPeriodInBlocks = MonthlyPeriodInBlocks;
	type QuarterlyPeriodInBlocks = QuarterlyPeriodInBlocks;
	type YearlyPeriodInBlocks = YearlyPeriodInBlocks;
}
*/

impl pallet_faucets::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Aura: pallet_aura::{Module, Config<T>},
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Module, Storage},
		Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>},
		Utility: pallet_utility::{Module, Call, Event},

		// Subsocial custom pallets:

		Permissions: pallet_permissions::{Module, Call},
		Posts: pallet_posts::{Module, Call, Storage, Event<T>},
		PostHistory: pallet_post_history::{Module, Storage},
		ProfileFollows: pallet_profile_follows::{Module, Call, Storage, Event<T>},
		Profiles: pallet_profiles::{Module, Call, Storage, Event<T>},
		ProfileHistory: pallet_profile_history::{Module, Storage},
		Reactions: pallet_reactions::{Module, Call, Storage, Event<T>},
		Roles: pallet_roles::{Module, Call, Storage, Event<T>},
		Scores: pallet_scores::{Module, Call, Storage, Event<T>},
		SpaceFollows: pallet_space_follows::{Module, Call, Storage, Event<T>},
		SpaceHistory: pallet_space_history::{Module, Storage},
		SpaceOwnership: pallet_space_ownership::{Module, Call, Storage, Event<T>},
		Spaces: pallet_spaces::{Module, Call, Storage, Event<T>, Config<T>},
		Utils: pallet_utils::{Module, Storage, Event<T>, Config<T>},

		// New experimental pallets. Not recommended to use in production yet.

		TippingLottery: pallet_tipping_lottery::{Module,Call ,  Storage,Event<T>},
		Faucets: pallet_faucets::{Module, Call, Storage, Event<T>},
		DotsamaClaims: pallet_dotsama_claims::{Module, Call, Storage, Event<T>},
		// SessionKeys: pallet_session_keys::{Module, Call, Storage, Event<T>},
		// Moderation: pallet_moderation::{Module, Call, Storage, Event<T>},
		// Donations: pallet_donations::{Module, Call, Storage, Event<T>},
		// Subscriptions: pallet_subscriptions::{Module, Call, Storage, Event<T>},
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    pallet_dotsama_claims::EnsureAllowedToClaimTokens<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllModules,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) ->
			Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> u64 {
			Aura::slot_duration()
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Module as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
					.to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
					.to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
					.to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
					.to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
					.to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_dotsama_claims, DotsamaClaims);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	impl space_follows_runtime_api::SpaceFollowsApi<Block, AccountId> for Runtime
    {
    	fn get_space_ids_followed_by_account(account: AccountId) -> Vec<SpaceId> {
    		SpaceFollows::get_space_ids_followed_by_account(account)
    	}

    	fn filter_followed_space_ids(account: AccountId, space_ids: Vec<SpaceId>) -> Vec<SpaceId> {
    		SpaceFollows::filter_followed_space_ids(account, space_ids)
    	}
    }

	impl spaces_runtime_api::SpacesApi<Block, AccountId, BlockNumber> for Runtime
	{
		fn get_spaces(start_id: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>> {
			Spaces::get_spaces(start_id, limit)
		}

		fn get_spaces_by_ids(space_ids: Vec<SpaceId>) -> Vec<FlatSpace<AccountId, BlockNumber>> {
			Spaces::get_spaces_by_ids(space_ids)
		}

		fn get_public_spaces(start_id: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>> {
			Spaces::get_public_spaces(start_id, limit)
		}

		fn get_unlisted_spaces(start_id: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>> {
			Spaces::get_unlisted_spaces(start_id, limit)
		}

		fn get_space_id_by_handle(handle: Vec<u8>) -> Option<SpaceId> {
			Spaces::get_space_id_by_handle(handle)
		}

        fn get_space_by_handle(handle: Vec<u8>) -> Option<FlatSpace<AccountId, BlockNumber>> {
        	Spaces::get_space_by_handle(handle)
        }

        fn get_public_space_ids_by_owner(owner: AccountId) -> Vec<SpaceId> {
        	Spaces::get_public_space_ids_by_owner(owner)
        }

        fn get_unlisted_space_ids_by_owner(owner: AccountId) -> Vec<SpaceId> {
        	Spaces::get_unlisted_space_ids_by_owner(owner)
        }

        fn get_next_space_id() -> SpaceId {
        	Spaces::get_next_space_id()
        }
    }

    impl posts_runtime_api::PostsApi<Block, AccountId, BlockNumber> for Runtime
    {
		fn get_posts_by_ids(post_ids: Vec<PostId>, offset: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>> {
			Posts::get_posts_by_ids(post_ids, offset, limit)
		}

		fn get_public_posts(kind_filter: Vec<FlatPostKind>, start_id: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>> {
			Posts::get_public_posts(kind_filter, start_id, limit)
		}

		fn get_public_posts_by_space_id(space_id: SpaceId, offset: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>> {
			Posts::get_public_posts_by_space_id(space_id, offset, limit)
		}

		fn get_unlisted_posts_by_space_id(space_id: SpaceId, offset: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>> {
			Posts::get_unlisted_posts_by_space_id(space_id, offset, limit)
		}

		fn get_reply_ids_by_parent_id(parent_id: PostId) -> Vec<PostId> {
			Posts::get_reply_ids_by_parent_id(parent_id)
		}

		fn get_reply_ids_by_parent_ids(parent_ids: Vec<PostId>) -> BTreeMap<PostId, Vec<PostId>> {
			Posts::get_reply_ids_by_parent_ids(parent_ids)
		}

		fn get_replies_by_parent_id(parent_id: PostId, offset: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>> {
			Posts::get_replies_by_parent_id(parent_id, offset, limit)
		}

		fn get_replies_by_parent_ids(parent_ids: Vec<PostId>, offset: u64, limit: u16) -> RepliesByPostId<AccountId, BlockNumber> {
			Posts::get_replies_by_parent_ids(parent_ids, offset, limit)
		}

		fn get_public_post_ids_by_space_id(space_id: SpaceId) -> Vec<PostId> {
			Posts::get_public_post_ids_by_space_id(space_id)
		}

		fn get_unlisted_post_ids_by_space_id(space_id: SpaceId) -> Vec<PostId> {
			Posts::get_unlisted_post_ids_by_space_id(space_id)
		}

		fn get_next_post_id() -> PostId {
			Posts::get_next_post_id()
		}

		fn get_feed(account: AccountId, offset: u64, limit: u16) -> Vec<FlatPost<AccountId, BlockNumber>> {
			Posts::get_feed(account, offset, limit)
		}
    }

	impl profile_follows_runtime_api::ProfileFollowsApi<Block, AccountId> for Runtime
    {
    	fn filter_followed_accounts(account: AccountId, maybe_following: Vec<AccountId>) -> Vec<AccountId> {
    		ProfileFollows::filter_followed_accounts(account, maybe_following)
    	}
    }

	impl profiles_runtime_api::ProfilesApi<Block, AccountId, BlockNumber> for Runtime
	{
		fn get_social_accounts_by_ids(
            account_ids: Vec<AccountId>
        ) -> Vec<FlatSocialAccount<AccountId, BlockNumber>> {
        	Profiles::get_social_accounts_by_ids(account_ids)
        }
	}

    impl reactions_runtime_api::ReactionsApi<Block, AccountId, BlockNumber> for Runtime
    {
		fn get_reactions_by_ids(reaction_ids: Vec<ReactionId>) -> Vec<FlatReaction<AccountId, BlockNumber>> {
			Reactions::get_reactions_by_ids(reaction_ids)
		}

		fn get_reactions_by_post_id(
			post_id: PostId,
			limit: u64,
			offset: u64
		) -> Vec<FlatReaction<AccountId, BlockNumber>> {
			Reactions::get_reactions_by_post_id(post_id, limit, offset)
		}

		fn get_reaction_kinds_by_post_ids_and_reactor(
			post_ids: Vec<PostId>,
        	reactor: AccountId,
		) -> BTreeMap<PostId, ReactionKind> {
			Reactions::get_reaction_kinds_by_post_ids_and_reactor(post_ids, reactor)
		}
    }

	impl roles_runtime_api::RolesApi<Block, AccountId> for Runtime
	{
		fn get_space_permissions_by_account(
			account: AccountId,
			space_id: SpaceId
		) -> Vec<SpacePermission> {
			Roles::get_space_permissions_by_account(account, space_id)
		}

		fn get_accounts_with_any_role_in_space(space_id: SpaceId) -> Vec<AccountId> {
			Roles::get_accounts_with_any_role_in_space(space_id)
		}

        fn get_space_ids_for_account_with_any_role(account_id: AccountId) -> Vec<SpaceId> {
			Roles::get_space_ids_for_account_with_any_role(account_id)
        }
	}
}
