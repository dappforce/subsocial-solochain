//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_std::{
	prelude::*,
	iter::FromIterator,
};
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys, MultiSignature,
	transaction_validity::{TransactionValidity, TransactionSource}, Perquintill, FixedPointNumber,
};
use sp_runtime::traits::{
	BlakeTwo256, Block as BlockT, IdentityLookup, Verify, IdentifyAccount, NumberFor, Saturating,
};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use timestamp::Call as TimestampCall;
pub use balances::Call as BalancesCall;
pub use sp_runtime::{Permill, Perbill};
pub use frame_support::{
	construct_runtime, parameter_types, StorageValue,
	traits::{KeyOwnerProofSystem, Randomness, Currency, Imbalance, OnUnbalanced, Filter},
	weights::{
		Weight, IdentityFee,
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
	},
};
use transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
pub use transaction_payment::{Multiplier, TargetedFeeAdjustment};

use pallet_permissions::{
	SpacePermission as SP,
	SpacePermissions,
	SpacePermissionSet
};
use pallet_utils::SpaceId;

pub mod constants;
use constants::{currency::*, time::*};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

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
	spec_version: 7,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
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

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	/// Assume 10% of weight for average on_initialize calls.
	pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
		.saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
	pub const Version: RuntimeVersion = VERSION;
}

impl system::Trait for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = BaseFilter;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = IdentityLookup<AccountId>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Maximum weight of each block.
	type MaximumBlockWeight = MaximumBlockWeight;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The weight of the overhead invoked on the block import process, independent of the
	/// extrinsics included in that block.
	type BlockExecutionWeight = BlockExecutionWeight;
	/// The base weight of any extrinsic processed by the runtime, independent of the
	/// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
	type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
	/// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
	/// idependent of the logic of that extrinsics. (Roughly max block weight - average on
	/// initialize cost).
	type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
	/// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
	type MaximumBlockLength = MaximumBlockLength;
	/// Portion of the block weight that is available to all normal transactions.
	type AvailableBlockRatio = AvailableBlockRatio;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type ModuleToIndex = ModuleToIndex;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = balances::AccountData<Balance>;
}

impl aura::Trait for Runtime {
	type AuthorityId = AuraId;
}

impl grandpa::Trait for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = ();

	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;

	type HandleEquivocation = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1 * CENTS;
}

impl balances::Trait for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = system::Module<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1 * MILLICENTS;
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl transaction_payment::Trait for Runtime {
    type Currency = Balances;
    type OnTransactionPayment = DealWithFees;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate =
    TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

impl sudo::Trait for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
}

impl pallet_scheduler::Trait for Runtime {
	type Event = Event;
	type Origin = Origin;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
}

impl pallet_utility::Trait for Runtime {
	type Event = Event;
	type Call = Call;
}

// Subsocial custom pallets go below:
// ------------------------------------------------------------------------------------------------

parameter_types! {
  pub const MinHandleLen: u32 = 5;
  pub const MaxHandleLen: u32 = 50;
}

impl pallet_utils::Trait for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MinHandleLen = MinHandleLen;
	type MaxHandleLen = MaxHandleLen;
}

parameter_types! {
  pub DefaultSpacePermissions: SpacePermissions = SpacePermissions {

    // No permissions disabled by default
    none: None,

    everyone: Some(SpacePermissionSet::from_iter(vec![
			SP::UpdateOwnSubspaces,
			SP::DeleteOwnSubspaces,
			SP::HideOwnSubspaces,

			SP::UpdateOwnPosts,
			SP::DeleteOwnPosts,
			SP::HideOwnPosts,

			SP::CreateComments,
			SP::UpdateOwnComments,
			SP::DeleteOwnComments,
			SP::HideOwnComments,

			SP::Upvote,
			SP::Downvote,
			SP::Share,
    ].into_iter())),

    // Followers can do everything that everyone else can.
    follower: None,

    space_owner: Some(SpacePermissionSet::from_iter(vec![
      SP::ManageRoles,
      SP::RepresentSpaceInternally,
      SP::RepresentSpaceExternally,
      SP::OverrideSubspacePermissions,
      SP::OverridePostPermissions,

      SP::CreateSubspaces,
      SP::CreatePosts,

      SP::UpdateSpace,
      SP::UpdateAnySubspace,
      SP::UpdateAnyPost,

      SP::DeleteAnySubspace,
      SP::DeleteAnyPost,

      SP::HideAnySubspace,
      SP::HideAnyPost,
      SP::HideAnyComment,

      SP::SuggestEntityStatus,
      SP::UpdateEntityStatus,

      SP::UpdateSpaceSettings,
    ].into_iter())),
  };
}

impl pallet_permissions::Trait for Runtime {
	type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
  pub const MaxCommentDepth: u32 = 10;
}

impl pallet_posts::Trait for Runtime {
	type Event = Event;
	type MaxCommentDepth = MaxCommentDepth;
	type PostScores = Scores;
	type AfterPostUpdated = PostHistory;
	type IsPostBlocked = ()/*Moderation*/;
}

parameter_types! {}

impl pallet_post_history::Trait for Runtime {}

parameter_types! {}

impl pallet_profile_follows::Trait for Runtime {
	type Event = Event;
	type BeforeAccountFollowed = Scores;
	type BeforeAccountUnfollowed = Scores;
}

parameter_types! {}

impl pallet_profiles::Trait for Runtime {
	type Event = Event;
	type AfterProfileUpdated = ProfileHistory;
}

parameter_types! {}

impl pallet_profile_history::Trait for Runtime {}

parameter_types! {}

impl pallet_reactions::Trait for Runtime {
	type Event = Event;
	type PostReactionScores = Scores;
}

parameter_types! {
  pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
}

impl pallet_roles::Trait for Runtime {
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

impl pallet_scores::Trait for Runtime {
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

impl pallet_space_follows::Trait for Runtime {
	type Event = Event;
	type BeforeSpaceFollowed = Scores;
	type BeforeSpaceUnfollowed = Scores;
}

parameter_types! {}

impl pallet_space_ownership::Trait for Runtime {
	type Event = Event;
}

parameter_types! {
	pub SpaceCreationFee: Balance = 50 * CENTS;
	pub const DefaultRPCLimit: u64 = 20;
}

impl pallet_spaces::Trait for Runtime {
	type Event = Event;
	type Roles = Roles;
	type SpaceFollows = SpaceFollows;
	type BeforeSpaceCreated = SpaceFollows;
	type AfterSpaceUpdated = SpaceHistory;
	type IsAccountBlocked = ()/*Moderation*/;
	type IsContentBlocked = ()/*Moderation*/;
	type SpaceCreationFee = SpaceCreationFee;
	type DefaultRPCLimit = DefaultRPCLimit;
}

parameter_types! {}

impl pallet_space_history::Trait for Runtime {}

pub struct BaseFilter;
impl Filter<Call> for BaseFilter {
	fn filter(c: &Call) -> bool {
		let is_set_balance = matches!(c, Call::Balances(balances::Call::set_balance(..)));
		let is_force_transfer = matches!(c, Call::Balances(balances::Call::force_transfer(..)));
		match *c {
			Call::Balances(..) => is_set_balance || is_force_transfer,
			_ => true,
		}
	}
}


parameter_types! {
	pub const MaxSessionKeysPerAccount: u16 = 10;
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

/*impl session_keys::Trait for Runtime {
	type Event = Event;
	type Call = Call;
	type MaxSessionKeysPerAccount = MaxSessionKeysPerAccount;
	type BaseFilter = SessionKeysProxyFilter;
}

impl pallet_donations::Trait for Runtime {
	type Event = Event;
}*/


parameter_types! {
	pub const DefaultAutoblockThreshold: u16 = 20;
}

/*
impl pallet_moderation::Trait for Runtime {
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

impl pallet_subscriptions::Trait for Runtime {
	type Event = Event;
	type Subscription = Call;
	type Scheduler = Scheduler;
	type DailyPeriodInBlocks = DailyPeriodInBlocks;
	type WeeklyPeriodInBlocks = WeeklyPeriodInBlocks;
	type MonthlyPeriodInBlocks = MonthlyPeriodInBlocks;
	type QuarterlyPeriodInBlocks = QuarterlyPeriodInBlocks;
	type YearlyPeriodInBlocks = YearlyPeriodInBlocks;
}

impl pallet_faucet::Trait for Runtime {
	type Event = Event;
	type Currency = Balances;
}
*/

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: system::{Module, Call, Config, Storage, Event<T>},
		RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},
		Timestamp: timestamp::{Module, Call, Storage, Inherent},
		Aura: aura::{Module, Config<T>, Inherent(Timestamp)},
		Grandpa: grandpa::{Module, Call, Storage, Config, Event},
		Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: transaction_payment::{Module, Storage},
		Sudo: sudo::{Module, Call, Config<T>, Storage, Event<T>},
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

		// Faucet: pallet_faucet::{Module, Call, Storage, Event<T>},
		// SessionKeys: session_keys::{Module, Call, Storage, Event<T>},
		// Moderation: pallet_moderation::{Module, Call, Storage, Event<T>},
		// Donations: pallet_donations::{Module, Call, Storage, Event<T>},
		// Subscriptions: pallet_subscriptions::{Module, Call, Storage, Event<T>},
	}
);

/// The address format for describing accounts.
pub type Address = AccountId;
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
	system::CheckSpecVersion<Runtime>,
	system::CheckTxVersion<Runtime>,
	system::CheckGenesis<Runtime>,
	system::CheckEra<Runtime>,
	system::CheckNonce<Runtime>,
	system::CheckWeight<Runtime>,
	transaction_payment::ChargeTransactionPayment<Runtime>
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

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

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
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

	impl transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
		UncheckedExtrinsic,
	> for Runtime {
		fn query_info(uxt: UncheckedExtrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
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

		fn submit_report_equivocation_extrinsic(
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

	impl spaces_runtime_api::SpacesApi<Block> for Runtime
	{
        fn get_last_space_id() -> SpaceId {
            Spaces::get_last_space_id()
        }

        fn get_hidden_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId> {
        	Spaces::get_hidden_space_ids(limit_opt, offset_opt)
        }

        fn get_public_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId> {
        	Spaces::get_public_space_ids(limit_opt, offset_opt)
        }

        fn get_unlisted_space_ids(limit_opt: Option<u64>, offset_opt: Option<u64>) -> Vec<SpaceId> {
        	Spaces::get_unlisted_space_ids(limit_opt, offset_opt)
        }
    }
}
