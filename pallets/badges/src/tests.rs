// Tests to be written here

use crate::{mock::*};
use crate::*;
use frame_support::{assert_ok, assert_noop};

use pallet_utils::{Error as UtilsError};
use pallet_spaces::{Error as SpacesError};

#[test]
fn create_badge_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());

        assert_eq!(BadgeForTest::next_badge_id(), BADGEID2);

        let badge = BadgeForTest::badge_by_id(BADGEID1).unwrap();
        assert_eq!(badge.created.account, ACCOUNT1);
        assert_eq!(badge.content, default_badge_content_ipfs());
        assert_eq!(badge.space_id, SPACE1);
        assert!(badge.updated.is_none());
    });
}

#[test]
fn create_badge_should_fail_with_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _create_badge(
                None,
                Some(SPACE2),
                None
            ), SpacesError::<Test>::SpaceNotFound
        );
    });
}

#[test]
fn create_badge_should_fail_with_no_permission() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _create_badge(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1),
                None
            ), Error::<Test>::NoPermissionToManageBadges
        );
    });
}

#[test]
fn create_badge_should_fail_with_ipfs_is_incorrect() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _create_badge(
                None,
                None,
                Some(self::invalid_badge_content_ipfs())
            ), UtilsError::<Test>::InvalidIpfsCid
        );
    });
}

//------------------------------------------------------------------
#[test]
fn update_badge_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_update_default_badge());

        let badge = BadgeForTest::badge_by_id(BADGEID1).unwrap();
        assert_eq!(badge.created.account, ACCOUNT1);
        assert_eq!(badge.content, updated_badge_content_ipfs());
    });
}

#[test]
fn update_badge_should_fail_with_badge_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_noop!(_update_badge(
                None,
                Some(BADGEID2),
                None,
            ), Error::<Test>::BadgeNotFound
        );
    });
}

#[test]
fn update_badge_should_fail_with_space_not_found() {
    ExtBuilder::build_with_badge_no_space().execute_with(|| {
        assert_noop!(_update_default_badge(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn update_badge_should_fail_with_no_permission() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_noop!(_update_badge(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
            ), Error::<Test>::NoPermissionToManageBadges
        );
    });
}

#[test]
fn update_badge_should_fail_with_ipfs_is_incorrect() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_noop!(_update_badge(
                None,
                None,
                Some(invalid_badge_content_ipfs()),
            ), UtilsError::<Test>::InvalidIpfsCid
        );
    });
}
//--------------------------------------------------------------

#[test]
fn delete_badge_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_delete_default_badge());

        assert!(BadgeForTest::badge_by_id(BADGEID1).is_none());
    });
}

#[test]
fn delete_badge_should_fail_with_badge_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_noop!(
            _delete_badge(
                None,
                Some(BADGEID2)
            ), Error::<Test>::BadgeNotFound
        );
    });
}

#[test]
fn delete_badge_should_fail_with_space_not_found() {
    ExtBuilder::build_with_badge_no_space().execute_with(|| {
        assert_noop!(_delete_default_badge(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn delete_badge_should_fail_with_no_permission() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_noop!(
            _delete_badge(
                Some(Origin::signed(ACCOUNT2)),
                None
            ), Error::<Test>::NoPermissionToManageBadges
        );
    });
}
//--------------------------------------------------------------

#[test]
fn award_badge_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_award_default_badge());

        assert_eq!(BadgeForTest::next_space_award_id(), SPACEAWARDID2);

        let space_award = BadgeForTest::space_award_by_id(SPACEAWARDID1).unwrap();
        assert_eq!(space_award.badge_id, BADGEID1);
        assert_eq!(space_award.created.account, ACCOUNT1);
        assert_eq!(space_award.recipient, SPACE2);
        assert_eq!(space_award.accepted, false);
    });
}

#[test]
fn award_badge_should_fail_with_badge_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_noop!(
            _award_badge(
                None,
                Some(BADGEID2),
                None,
                None
            ), Error::<Test>::BadgeNotFound
        );
    });
}

#[test]
fn award_badge_should_fail_with_space_not_found() {
    ExtBuilder::build_with_badge_no_space().execute_with(|| {
        assert_noop!(_award_default_badge(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn award_badge_should_fail_with_no_permission() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_noop!(
            _award_badge(
                Some(Origin::signed(ACCOUNT2)),
                None,
                None,
                None
            ), Error::<Test>::NoPermissionToManageAwards
        );
    });
}
//--------------------------------------------------------------

#[test]
fn accept_award_should_work() {
    ExtBuilder::build_with_two_spaces().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_award_default_badge());
        assert_ok!(_accept_default_award());

        let space_award = BadgeForTest::space_award_by_id(SPACEAWARDID1).unwrap();
        assert_eq!(space_award.accepted, true);
    });
}

#[test]
fn accept_award_should_fail_with_space_award_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_award_default_badge());
        assert_noop!(
            _accept_award(
                None,
                Some(SPACEAWARDID2)
            ), Error::<Test>::SpaceAwardNotFound
        );
    });
}

#[test]
fn accept_award_should_fail_with_space_not_found() {
    ExtBuilder::build_with_badge_no_space().execute_with(|| {
        assert_noop!(_accept_default_award(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn accept_award_should_fail_with_no_permission() {
    ExtBuilder::build_with_two_spaces().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_award_default_badge());
        assert_noop!(
            _accept_award(
                Some(Origin::signed(ACCOUNT2)),
                None
            ), Error::<Test>::NoPermissionToManageAwards
        );
    });
}
//--------------------------------------------------------------

#[test]
fn delete_badge_award_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_award_default_badge());
        assert_ok!(_delete_default_badge_award());

        assert!(BadgeForTest::space_award_by_id(SPACEAWARDID1).is_none());
    });
}

#[test]
fn delete_badge_award_should_fail_with_space_award_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_award_default_badge());
        assert_noop!(
            _delete_badge_award(
                None,
                Some(SPACEAWARDID2)
            ), Error::<Test>::SpaceAwardNotFound
        );
    });
}

#[test]
fn delete_badge_award_should_fail_with_space_not_found() {
    ExtBuilder::build_with_badge_no_space().execute_with(|| {
        assert_noop!(_delete_default_badge_award(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn delete_badge_award_should_fail_with_no_permission() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_badge());
        assert_ok!(_award_default_badge());
        assert_noop!(
            _delete_badge_award(
                Some(Origin::signed(ACCOUNT2)),
                None
            ), Error::<Test>::NoPermissionToManageAwards
        );
    });
}