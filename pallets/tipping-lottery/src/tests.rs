use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::One;

#[test]
fn votes_increase_and_registered() {
	ExtBuilder::build_with_space_and_post().execute_with(|| {
		let block_number = System::block_number();
		init_account_with_balance(&ACCOUNT_SCOPE_OWNER);
		assert_ok!(Lottery::vote_for_post(Origin::signed(ACCOUNT_SCOPE_OWNER), POST1, 2120));

		let (exists, current_lottery_id) = Lottery::lottery_exists(block_number);
		assert_eq!(exists, true);

		let number_of_votes = Lottery::post_votes_number_of_lottery((current_lottery_id, POST1));
		assert_eq!(number_of_votes, 2120);

		assert_ok!(Lottery::vote_for_post(Origin::signed(ACCOUNT_SCOPE_OWNER), POST1, 1000));
		let number_of_votes = Lottery::post_votes_number_of_lottery((current_lottery_id, POST1));
		assert_eq!(number_of_votes, 1000 + 2120);
		let votes = Lottery::votes_for_lottery(current_lottery_id);
		assert_eq!(votes.len(), 2)
	})
}

#[test]
fn end_lottery() {
	ExtBuilder::build_and_vote().execute_with(|| {
		let block_number = System::block_number();
		let (exists, current_lottery_id) = Lottery::lottery_exists(block_number);
		assert_eq!(exists, true);
		let lottery_status = Lottery::lottery_status_by_Lottery_id(current_lottery_id);
		assert_eq!(lottery_status.is_done(), false);
		let next_lottery_id = current_lottery_id + <<Test as frame_system::Trait>::BlockNumber as One>::one();
		Lottery::end_previous_lottery(next_lottery_id);
		let lottery_status = Lottery::lottery_status_by_Lottery_id(current_lottery_id);
		assert_eq!(lottery_status.is_done(), true);
	})
}

#[test]
fn creator_balance_increase() {
	ExtBuilder::build_with_space_and_post().execute_with(|| {
		let block_number = System::block_number();
		init_account_with_balance(&ACCOUNT_SCOPE_OWNER);
		init_account_with_balance(&ACCOUNT_NOT_MODERATOR);

		let creator_balance_before_post_win = Balances::free_balance(&ACCOUNT_SCOPE_OWNER);
		assert_ok!(Lottery::vote_for_post(
			Origin::signed(ACCOUNT_NOT_MODERATOR),
			POST1,
			500000
		));

		let voter_balance_before_before_win = Balances::free_balance(&ACCOUNT_NOT_MODERATOR);
		let next_lottery_id = block_number + <<Test as frame_system::Trait>::BlockNumber as One>::one();
		Lottery::end_previous_lottery(next_lottery_id);
		let creator_balance_after_post_win = Balances::free_balance(&ACCOUNT_SCOPE_OWNER);
		let voter_balance_before_voter_win = Balances::free_balance(&ACCOUNT_NOT_MODERATOR);

		assert_eq!(creator_balance_before_post_win < creator_balance_after_post_win, true);
		assert_eq!(voter_balance_before_before_win < voter_balance_before_voter_win, true);
		dbg!(creator_balance_before_post_win);
		dbg!(creator_balance_after_post_win);
	})
}
