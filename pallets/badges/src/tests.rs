// Tests to be written here

use crate::{mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn create_badge_should_work() {
	ExtBuilder::build_with_space().execute_with(|| {
		assert_ok!(_create_default_badge());

		assert_eq!(Badge::next_badge_id(), BADGEID2);

		let badge = Badge::badge_by_id(BADGEID1).unwrap();
		assert_eq!(badge.created.account, ACCOUNT1);
		assert_eq!(badge.content, default_badge_content_ipfs());
		assert_eq!(badge.space_id, SPACE1);
		assert!(badge.updated.is_none());
	});
}

/*
create_badge_should_fail_with_no_permission
*/