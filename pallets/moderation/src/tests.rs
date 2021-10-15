use crate::{Error, mock::*};
use crate::*;

use frame_support::{assert_ok, assert_noop};
use pallet_posts::PostById;
use pallet_utils::{
    Error as UtilsError,
    mock_functions::invalid_content_ipfs,
};
use pallet_spaces::{SpaceById, Error as SpaceError};

#[test]
fn report_entity_should_work() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_eq!(Moderation::next_report_id(), REPORT2);

        let report = Moderation::report_by_id(REPORT1).unwrap();
        assert_eq!(report.id, REPORT1);
        assert_eq!(report.created.account, ACCOUNT_SCOPE_OWNER);
        assert_eq!(report.reported_entity, EntityId::Post(POST1));
        assert_eq!(report.reported_within, SPACE1);
        assert_eq!(report.reason, valid_content_ipfs());
    });
}

#[test]
fn report_entity_should_fail_when_no_reason_provided() {
    ExtBuilder::build_with_space_and_post().execute_with(|| {
        assert_noop!(
            _report_entity(
                None,
                None,
                None,
                Some(Content::None)
            ), Error::<Test>::ReasonIsEmpty
        );
    });
}

#[test]
fn report_entity_should_fail_when_reason_is_invalid_ipfs_cid() {
    ExtBuilder::build_with_space_and_post().execute_with(|| {
        assert_noop!(
            _report_entity(
                None,
                None,
                None,
                Some(invalid_content_ipfs())
            ), UtilsError::<Test>::InvalidIpfsCid
        );
    });
}

#[test]
fn report_entity_should_fail_when_invalid_scope_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_report_default_post(), Error::<Test>::ScopeNotFound);
    });
}

#[test]
fn report_entity_should_fail_when_entity_already_reported() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_noop!(_report_default_post(), Error::<Test>::AlreadyReportedEntity);
    });
}

// Suggest entity status
//-------------------------------------------------------------------------

#[test]
fn suggest_entity_status_should_work() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_ok!(_suggest_blocked_status_for_post());

        let suggestions = Moderation::suggested_statuses(EntityId::Post(POST1), SPACE1);
        let expected_status = SuggestedStatus::<Test>::new(
            ACCOUNT_SCOPE_OWNER,
            Some(EntityStatus::Blocked),
            Some(REPORT1),
        );

        assert!(suggestions == vec![expected_status]);
    });
}

#[test]
fn suggest_entity_status_should_fail_when_report_not_found() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_noop!(
            _suggest_entity_status(
                None,
                None,
                None,
                None,
                Some(Some(REPORT2))
            ), Error::<Test>::ReportNotFound
        );
    });
}

#[test]
fn suggest_entity_status_should_fail_when_report_in_another_scope() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_noop!(
            _suggest_entity_status(
                None,
                None,
                Some(SPACE2),
                None,
                None
            ), Error::<Test>::SuggestedStatusInWrongScope
        );
    });
}

#[test]
fn suggest_entity_status_should_fail_when_same_entity_status_already_suggested() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_ok!(_suggest_blocked_status_for_post());
        assert_ok!(_update_post_status_to_allowed());
        assert_noop!(
            _suggest_entity_status(
                None,
                None,
                None,
                Some(Some(EntityStatus::Allowed)),
                None
            ), Error::<Test>::SuggestedSameEntityStatus
        );
    });
}

#[test]
fn suggest_entity_status_should_fail_when_scope_not_found() {
    ExtBuilder::build_with_report_then_remove_scope().execute_with(|| {
        assert_noop!(_suggest_blocked_status_for_post(), Error::<Test>::ScopeNotFound);
    });
}

#[test]
fn suggest_entity_status_should_fail_when_origin_has_no_permission() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_noop!(
            _suggest_entity_status(
                Some(Origin::signed(ACCOUNT_NOT_MODERATOR)),
                None,
                None,
                None,
                None
            ), Error::<Test>::NoPermissionToSuggestEntityStatus
        );
    });
}

#[test]
fn suggest_entity_status_should_autoblock_and_kick_entity_when_threshold_reached() {
    ExtBuilder::build_with_report_then_grant_role_to_suggest_entity_status().execute_with(|| {
        let space_before_autoblock = Spaces::<Test>::space_by_id(SPACE1).unwrap();
        let post_before_autoblock = Posts::post_by_id(POST1).unwrap();

        assert!(space_before_autoblock.posts_count == 1);
        assert!(post_before_autoblock.space_id == Some(SPACE1));
        assert_eq!(Posts::post_ids_by_space_id(SPACE1), vec![POST1]);

        // All accounts that have the corresponding role suggest entity status 'Blocked'.
        let accs = moderators();
        for (i, acc) in accs.into_iter().enumerate() {
            let res = _suggest_entity_status(Some(Origin::signed(acc)), None, None, None, None);
            if (i as u16) < DefaultAutoblockThreshold::get() {
                assert_ok!(res);
            } else {
                assert_noop!(res, Error::<Test>::SuggestedSameEntityStatus);
            }
        }

        let space_after_autoblock = Spaces::<Test>::space_by_id(SPACE1).unwrap();
        let post_after_autoblock = Posts::post_by_id(POST1).unwrap();

        assert!(space_after_autoblock.posts_count == 0);
        assert!(post_after_autoblock.space_id.is_none());
        assert!(Posts::post_ids_by_space_id(SPACE1).is_empty());
    });
}

// Update entity status
//----------------------------------------------------------------------------

#[test]
fn update_entity_status_should_work_for_status_allowed() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_ok!(_suggest_blocked_status_for_post());
        assert_ok!(_update_post_status_to_allowed());

        let status = Moderation::status_by_entity_in_space(EntityId::Post(POST1), SPACE1).unwrap();
        assert_eq!(status, EntityStatus::Allowed);
    });
}

#[test]
fn update_entity_status_should_work_for_status_blocked() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_ok!(_suggest_blocked_status_for_post());
        assert_ok!(
            _update_entity_status(
                None,
                None,
                None,
                Some(Some(EntityStatus::Blocked))
            )
        );

        // Check that post was removed from its space,
        // because when removing a post, we set its space to None
        let post = PostById::<Test>::get(POST1).unwrap();
        assert!(post.space_id.is_none());
    });
}

#[test]
fn update_entity_status_should_fail_when_invalid_scope_provided() {
    ExtBuilder::build_with_report_then_remove_scope().execute_with(|| {
        assert_noop!(_update_post_status_to_allowed(), Error::<Test>::ScopeNotFound);
    });
}

#[test]
fn update_entity_status_should_fail_when_origin_has_no_permission() {
    ExtBuilder::build_with_space_and_post().execute_with(|| {
        assert_noop!(
            _update_entity_status(
                Some(Origin::signed(ACCOUNT_NOT_MODERATOR)),
                None,
                None,
                None
            ), Error::<Test>::NoPermissionToUpdateEntityStatus
        );
    });
}

// Delete entity status
//---------------------------------------------------------------------------

#[test]
fn delete_entity_status_should_work() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_ok!(_suggest_blocked_status_for_post());
        assert_ok!(_update_post_status_to_allowed());
        assert_ok!(_delete_post_status());

        let status = Moderation::status_by_entity_in_space(EntityId::Post(POST1), SPACE1);
        assert!(status.is_none());
    });
}

#[test]
fn delete_entity_status_should_fail_when_entity_has_no_status_in_scope() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_noop!(_delete_post_status(), Error::<Test>::EntityHasNoStatusInScope);
    });
}

#[test]
fn delete_entity_status_should_fail_when_scope_not_found() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_ok!(_suggest_blocked_status_for_post());
        assert_ok!(_update_post_status_to_allowed());
        SpaceById::<Test>::remove(SPACE1);
        assert_noop!(_delete_post_status(), Error::<Test>::ScopeNotFound);
    });
}

#[test]
fn delete_entity_status_should_fail_when_origin_has_no_permission() {
    ExtBuilder::build_with_space_and_post_then_report().execute_with(|| {
        assert_ok!(_suggest_blocked_status_for_post());
        assert_ok!(_update_post_status_to_allowed());
        assert_noop!(
            _delete_entity_status(
                Some(Origin::signed(ACCOUNT_NOT_MODERATOR)),
                None,
                None
            ), Error::<Test>::NoPermissionToUpdateEntityStatus
        );
    });
}

// Update moderation settings
//----------------------------------------------------------------------------

#[test]
fn update_moderation_settings_should_work() {
    ExtBuilder::build_with_space_and_post().execute_with(|| {
        assert_ok!(_update_autoblock_threshold_in_moderation_settings());

        let settings = Moderation::moderation_settings(SPACE1).unwrap();
        assert_eq!(settings.autoblock_threshold, Some(AUTOBLOCK_THRESHOLD));
    });
}

// TODO test that autoblock works

#[test]
fn update_moderation_settings_should_fail_when_no_updates_provided() {
    ExtBuilder::build_with_space_and_post().execute_with(|| {
        assert_noop!(
            _update_moderation_settings(
                None,
                None,
                Some(empty_moderation_settings_update())
            ), Error::<Test>::NoUpdatesForModerationSettings
        );
    });
}

#[test]
fn update_moderation_settings_should_fail_when_space_not_found() {
    ExtBuilder::build_with_report_then_remove_scope().execute_with(|| {
        assert_noop!(
            _update_autoblock_threshold_in_moderation_settings(),
            SpaceError::<Test>::SpaceNotFound
        );
    });
}

#[test]
fn update_moderation_settings_should_fail_when_origin_has_no_permission() {
    ExtBuilder::build_with_space_and_post().execute_with(|| {
        assert_noop!(
            _update_moderation_settings(
                Some(Origin::signed(ACCOUNT_NOT_MODERATOR)),
                None,
                None
            ), Error::<Test>::NoPermissionToUpdateModerationSettings
        );
    });
}
