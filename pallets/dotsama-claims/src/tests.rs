use crate::{Error, mock::*, EligibleAccounts};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::DispatchError::BadOrigin;

// Test `fn claim_tokens(..)`

#[test]
fn claim_tokens_should_work() {
    ExtBuilder::build_with_set_rewards_sender_and_eligible_accounts().execute_with(|| {
        let claimed_before_first_claim: Option<Balance> = DotsamaClaims::total_tokens_claimed();

        assert_ok!(_claim_tokens_by_account1());

        let claimed_after_first_claim: Option<Balance> = DotsamaClaims::total_tokens_claimed();

        assert_eq!(claimed_before_first_claim, None);
        assert_eq!(claimed_after_first_claim, Some(InitialClaimAmount::get()));
        assert_eq!(Balances::free_balance(ACCOUNT1), InitialClaimAmount::get());
        assert_eq!(DotsamaClaims::tokens_claimed_by_account(ACCOUNT1), InitialClaimAmount::get());
    });
}

#[test]
fn claim_tokens_should_fail_when_rewards_sender_not_set() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_claim_tokens_by_account1(), Error::<Test>::NoRewardsSenderSet);
    });
}

#[test]
fn claim_tokens_should_fail_when_rewards_sender_has_insufficient_balance() {
    ExtBuilder::build_with_set_rewards_sender_and_eligible_accounts().execute_with(|| {
        assert_ok!(_claim_tokens_by_account1());
        assert_noop!(_claim_tokens_by_account2(), Error::<Test>::RewardsSenderHasInsufficientBalance);
    });
}

#[test]
fn claim_tokens_should_fail_when_account_not_eligible_to_claim() {
    ExtBuilder::build_with_set_rewards_sender().execute_with(|| {
        assert_noop!(_claim_tokens_by_account1(), Error::<Test>::AccountNotEligible);
    });
}

#[test]
fn claim_tokens_should_fail_when_the_account_already_claimed_tokens() {
    ExtBuilder::build_with_set_rewards_sender_and_eligible_accounts().execute_with(|| {
        assert_ok!(_claim_tokens_by_account1());
        
        // The same account tries to claim tokens again:
        assert_noop!(_claim_tokens_by_account1(), Error::<Test>::TokensAlreadyClaimed);
    });
}

// Test `fn set_rewards_sender(..)`

#[test]
fn set_rewards_sender_should_work() {
    ExtBuilder::build().execute_with(|| {
        // By default, a rewards sender should not be set.
        assert_eq!(DotsamaClaims::rewards_sender(), None);

        // `set_rewards_sender_should_work`.
        assert_ok!(_set_default_rewards_sender());
        assert_eq!(DotsamaClaims::rewards_sender(), Some(REWARDS_SENDER));
    });
}

#[test]
fn replace_rewards_sender_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_set_default_rewards_sender());
        assert_ok!(_set_rewards_sender(None, Some(Some(ALT_REWARDS_SENDER))));
        assert_eq!(DotsamaClaims::rewards_sender(), Some(ALT_REWARDS_SENDER));
    });
}

#[test]
fn remove_rewards_sender_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_set_default_rewards_sender());
        assert_ok!(_set_rewards_sender(None, Some(None)));
        assert_eq!(DotsamaClaims::rewards_sender(), None);
    });
}

#[test]
fn set_rewards_sender_should_fail_when_origin_not_root() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_not_root_tries_to_set_rewards_sender(), BadOrigin);
    });
}

#[test]
fn set_rewards_sender_should_fail_when_it_has_insufficient_balance() {
    ExtBuilder::build_with_insufficient_balances_for_rewards_sender().execute_with(|| {
        assert_noop!(_set_default_rewards_sender(), Error::<Test>::RewardsSenderHasInsufficientBalance);
    });
}

// Test `fn add_eligible_accounts(..)`

#[test]
fn add_eligible_accounts_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_eq!(EligibleAccounts::<Test>::iter().count(), 0);

        let mut eligible_accounts = Vec::new();
        for account in 1..=AccountsSetLimit::get() as AccountId {
            eligible_accounts.push(account);
        }

        assert_ok!(_add_eligible_accounts(None, eligible_accounts.clone()));
        assert_eq!(EligibleAccounts::<Test>::iter().count(), AccountsSetLimit::get() as usize);

        for account in eligible_accounts {
            assert_eq!(DotsamaClaims::eligible_accounts(account), true);
        }
    });
}

#[test]
fn add_eligible_accounts_should_fail_when_origin_not_root() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_not_root_tries_to_add_eligible_accounts(), BadOrigin);
    });
}

#[test]
fn add_eligible_accounts_should_fail_when_trying_to_add_accounts_over_limit() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_add_eligible_accounts_over_limit(), Error::<Test>::AddingTooManyAccountsAtOnce);
    });
}
