pub use super::*;

use sp_core::H256;
use frame_support::{impl_outer_origin, assert_ok, assert_noop, parameter_types, weights::Weight, dispatch::DispatchResult};
use sp_runtime::{
  traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};

impl_outer_origin! {
  pub enum Origin for Test {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const MaximumBlockWeight: Weight = 1024;
  pub const MaximumBlockLength: u32 = 2 * 1024;
  pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl system::Trait for Test {
  type Origin = Origin;
  type Call = ();
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
  type MaximumBlockLength = MaximumBlockLength;
  type AvailableBlockRatio = AvailableBlockRatio;
  type Version = ();
  type ModuleToIndex = ();
}

parameter_types! {
  pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Trait for Test {
  type Moment = u64;
  type OnTimestampSet = ();
  type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
	pub const MinSpaceOwners: u16 = 1;
	pub const MaxSpaceOwners: u16 = 1000;
	pub const MaxChangeNotesLength: u16 = 1024;
	pub const BlocksToLive: u64 = 302_400;
	pub const DeleteExpiredChangesPeriod: u64 = 1800;
}

impl Trait for Test {
  type Event = ();
  type MinSpaceOwners = MinSpaceOwners;
  type MaxSpaceOwners = MaxSpaceOwners;
  type MaxChangeNotesLength = MaxChangeNotesLength;
  type BlocksToLive = BlocksToLive;
  type DeleteExpiredChangesPeriod = DeleteExpiredChangesPeriod;
}

type MultiOwnership = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
fn new_test_ext() -> sp_io::TestExternalities {
  system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

type AccountId = u64;

const ACCOUNT1: AccountId = 1;
const ACCOUNT2: AccountId = 2;
const ACCOUNT3: AccountId = 3;
const ACCOUNT4: AccountId = 4;

fn change_note() -> Vec<u8> {
  b"Default change proposal".to_vec()
}

fn _create_default_space_owners() -> DispatchResult {
  _create_space_owners(None, None, None, None)
}

fn _create_space_owners(
  origin: Option<Origin>,
  space_id: Option<SpaceId>,
  owners: Option<Vec<AccountId>>,
  threshold: Option<u16>,
) -> DispatchResult {
  MultiOwnership::create_space_owners(
    origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
    space_id.unwrap_or(1),
    owners.unwrap_or_else(|| vec![ACCOUNT1, ACCOUNT2]),
    threshold.unwrap_or(2),
  )
}

fn _propose_default_change() -> DispatchResult {
  _propose_change(None, None, None, None, None, None)
}

#[allow(clippy::option_option)]
fn _propose_change(
  origin: Option<Origin>,
  space_id: Option<SpaceId>,
  add_owners: Option<Vec<AccountId>>,
  remove_owners: Option<Vec<AccountId>>,
  new_threshold: Option<Option<u16>>,
  notes: Option<Vec<u8>>,
) -> DispatchResult {
  MultiOwnership::propose_change(
    origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
    space_id.unwrap_or(1),
    add_owners.unwrap_or_else(|| vec![ACCOUNT3]),
    remove_owners.unwrap_or_else(|| vec![]),
    new_threshold.unwrap_or(Some(3)),
    notes.unwrap_or_else(self::change_note),
  )
}

fn _confirm_default_change() -> DispatchResult {
  _confirm_change(None, None, None)
}

fn _confirm_change(
  origin: Option<Origin>,
  space_id: Option<SpaceId>,
  change_id: Option<ChangeId>,
) -> DispatchResult {
  MultiOwnership::confirm_change(
    origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
    space_id.unwrap_or(1),
    change_id.unwrap_or(1),
  )
}

fn _cancel_default_proposal() -> DispatchResult {
  _cancel_change(None, None, None)
}

fn _cancel_change(
  origin: Option<Origin>,
  space_id: Option<SpaceId>,
  change_id: Option<ChangeId>,
) -> DispatchResult {
  MultiOwnership::cancel_change(
    origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
    space_id.unwrap_or(1),
    change_id.unwrap_or(1),
  )
}

#[test]
fn create_space_owners_should_work() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());

    // Check storages
    let mut check: Vec<u64> = MultiOwnership::space_ids_owned_by_account_id(ACCOUNT1).iter().cloned().collect();
    assert_eq!(check, vec![1]);

    check = MultiOwnership::space_ids_owned_by_account_id(ACCOUNT2).iter().cloned().collect();
    assert_eq!(check, vec![1]);

    // Check whether data is stored correctly
    let space_owners = MultiOwnership::space_owners_by_space_id(1).unwrap();
    assert_eq!(space_owners.owners, vec![ACCOUNT1, ACCOUNT2]);
    assert_eq!(space_owners.space_id, 1);
    assert_eq!(space_owners.threshold, 2);
    assert_eq!(space_owners.changes_count, 0);
  });
}

// -------

#[test]
fn propose_change_should_work() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());

    // Check storages
    let set_to_vec: Vec<u64> = MultiOwnership::pending_change_ids().iter().cloned().collect();
    assert_eq!(set_to_vec, vec![1]);
    assert_eq!(MultiOwnership::pending_change_id_by_space_id(1), Some(1));
    assert_eq!(MultiOwnership::next_change_id(), 2);

    // Check whether data is stored correctly
    let change = MultiOwnership::change_by_id(1).unwrap();
    assert_eq!(change.add_owners, vec![ACCOUNT3]);
    assert_eq!(change.remove_owners, vec![]);
    assert_eq!(change.new_threshold, Some(3));
    assert_eq!(change.notes, self::change_note());
    assert_eq!(change.confirmed_by, vec![ACCOUNT1]);
  });
}

#[test]
fn propose_change_should_work_with_only_one_owner() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_change(
      None,
      None,
      Some(vec![ACCOUNT3]),
      Some(vec![ACCOUNT1, ACCOUNT2]),
      Some(Some(1)),
      None)
    );

    // Check storages
    assert_eq!(MultiOwnership::pending_change_id_by_space_id(1), Some(1));
    assert_eq!(MultiOwnership::next_change_id(), 2);

    // Check whether data is stored correctly
    let change = MultiOwnership::change_by_id(1).unwrap();
    assert_eq!(change.add_owners, vec![ACCOUNT3]);
    assert_eq!(change.remove_owners, vec![ACCOUNT1, ACCOUNT2]);
    assert_eq!(change.new_threshold, Some(1));
    assert_eq!(change.notes, self::change_note());
    assert_eq!(change.confirmed_by, vec![ACCOUNT1]);
  });
}

#[test]
fn propose_change_should_fail_zero_threshold() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_noop!(_propose_change(None, None, Some(vec![]), Some(vec![]), Some(Some(0)), None), Error::<Test>::ZeroThershold);
  });
}

#[test]
fn propose_change_should_fail_too_big_threshold() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_noop!(_propose_change(None, None, Some(vec![]), Some(vec![]), Some(Some(3)), None), Error::<Test>::TooBigThreshold);
  });
}

#[test]
fn propose_change_should_fail_no_owners_left() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_noop!(_propose_change(
      None,
      None,
      Some(vec![]),
      Some(vec![ACCOUNT1, ACCOUNT2]),
      Some(None),
      None
     ), Error::<Test>::NoSpaceOwnersLeft);
  });
}

#[test]
fn propose_change_should_fail_proposal_already_exist() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());
    assert_noop!(_propose_change(
      Some(Origin::signed(ACCOUNT2)),
      None, None, None, Some(None), None
     ), Error::<Test>::PendingChangeAlreadyExists);
  });
}

#[test]
fn propose_change_should_fail_no_updates_on_owners() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_noop!(_propose_change(
      None,
      None,
      Some(vec![]),
      Some(vec![ACCOUNT3]),
      Some(None),
      None
     ), Error::<Test>::NoFieldsUpdatedOnProposal);
  });
}

#[test]
fn propose_change_should_fail_no_updates_on_threshold() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_noop!(_propose_change(
      None,
      None,
      Some(vec![]),
      Some(vec![]),
      Some(Some(2)),
      None
     ), Error::<Test>::NoFieldsUpdatedOnProposal);
  });
}

#[test]
fn propose_change_should_fail_not_a_space_owner() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_noop!(_propose_change(
      Some(Origin::signed(ACCOUNT3)),
      None,
      Some(vec![]),
      Some(vec![]),
      Some(Some(2)),
      None
     ), Error::<Test>::NotASpaceOwner);
  });
}

// -------

#[test]
fn confirm_change_should_work_owner_added() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());
    assert_ok!(_confirm_default_change());

    // Check storages
    assert_eq!(MultiOwnership::pending_change_id_by_space_id(1), None);
    assert_eq!(MultiOwnership::executed_change_ids_by_space_id(1), vec![1]);
    assert_eq!(MultiOwnership::next_change_id(), 2);

    // Check whether data is stored correctly
    let change = MultiOwnership::change_by_id(1).unwrap();
    assert_eq!(change.confirmed_by, vec![ACCOUNT1, ACCOUNT2]);

    // Check whether updates applied
    let space_owners = MultiOwnership::space_owners_by_space_id(1).unwrap();
    assert_eq!(space_owners.owners, vec![ACCOUNT1, ACCOUNT2, ACCOUNT3]);
    assert_eq!(space_owners.threshold, 3);
  });
}

#[test]
fn confirm_change_should_work_owner_removed() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_change(
      None,
      None,
      Some(vec![]),
      Some(vec![ACCOUNT2]),
      Some(Some(1)),
      None
    ));
    assert_ok!(_confirm_default_change());

    // Check storages
    assert_eq!(MultiOwnership::pending_change_id_by_space_id(1), None);
    assert_eq!(MultiOwnership::executed_change_ids_by_space_id(1), vec![1]);
    assert_eq!(MultiOwnership::next_change_id(), 2);

    // Check whether data is stored correctly
    let change = MultiOwnership::change_by_id(1).unwrap();
    assert_eq!(change.confirmed_by, vec![ACCOUNT1, ACCOUNT2]);

    // Check whether updates applied
    let space_owners = MultiOwnership::space_owners_by_space_id(1).unwrap();
    assert_eq!(space_owners.owners, vec![ACCOUNT1]);
    assert_eq!(space_owners.threshold, 1);
  });
}

#[test]
fn confirm_change_should_fail_not_related_to_space_owners() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());
    assert_ok!(_create_space_owners(
      Some(Origin::signed(ACCOUNT3)),
      Some(2),
      Some(vec![ACCOUNT3]),
      Some(1)
    ));
    assert_ok!(_propose_change(
      Some(Origin::signed(ACCOUNT3)),
      Some(2),
      Some(vec![ACCOUNT1]),
      Some(vec![]),
      Some(Some(2)),
      Some(self::change_note())
    ));

    assert_noop!(_confirm_change(
      None,
      Some(1),
      Some(2)
    ), Error::<Test>::ChangeNotRelatedToSpace);
  });
}

#[test]
fn confirm_change_should_fail_already_confirmed() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_space_owners(
      Some(Origin::signed(ACCOUNT1)),
      Some(1),
      Some(vec![ACCOUNT1, ACCOUNT2, ACCOUNT4]),
      Some(3)
    ));
    assert_ok!(_propose_default_change());
    assert_ok!(_confirm_default_change());

    assert_noop!(_confirm_default_change(), Error::<Test>::ChangeAlreadyConfirmed);
  });
}

#[test]
fn confirm_change_should_fail_not_a_space_owner() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());
    assert_noop!(_confirm_change(
      Some(Origin::signed(ACCOUNT3)),
      None,
      None
     ), Error::<Test>::NotASpaceOwner);
  });
}

// -------

#[test]
fn cancel_proposal_should_work() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());
    assert_ok!(_cancel_default_proposal());

    // Check storages
    let set_to_vec: Vec<u64> = MultiOwnership::pending_change_ids().iter().cloned().collect();
    assert_eq!(set_to_vec, vec![]);
    assert_eq!(MultiOwnership::pending_change_id_by_space_id(1), None);
    assert_eq!(MultiOwnership::next_change_id(), 2);
    assert!(MultiOwnership::change_by_id(1).is_none());
  });
}

#[test]
fn cancel_proposal_should_fail_not_related_to_space_owners() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());
    assert_ok!(_create_space_owners(
      Some(Origin::signed(ACCOUNT3)),
      Some(2),
      Some(vec![ACCOUNT3]),
      Some(1)
    ));
    assert_ok!(_propose_change(
      Some(Origin::signed(ACCOUNT3)),
      Some(2),
      Some(vec![ACCOUNT1]),
      Some(vec![]),
      Some(Some(2)),
      Some(self::change_note())
    ));

    assert_noop!(_cancel_change(
      None,
      Some(1),
      Some(2)
    ), Error::<Test>::ChangeNotRelatedToSpace);
  });
}

#[test]
fn cancel_proposal_should_fail_not_a_creator() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());
    assert_noop!(_cancel_change(
      Some(Origin::signed(ACCOUNT2)),
      None,
      None
    ), Error::<Test>::NotAChangeCreator);
  });
}

#[test]
fn cancel_proposal_should_fail_not_a_space_owner() {
  new_test_ext().execute_with(|| {
    assert_ok!(_create_default_space_owners());
    assert_ok!(_propose_default_change());
    assert_noop!(_cancel_change(
      Some(Origin::signed(ACCOUNT3)),
      None,
      None
     ), Error::<Test>::NotASpaceOwner);
  });
}
