//! # Lottery Monetization Module
//!
//! The Monetization module by lottery method

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
	traits::Currency,
};
use frame_support::traits::Get;
use frame_system::{self as system, ensure_signed};
use log;
use sp_runtime::{
	RuntimeDebug,
	sp_std::convert::TryInto,
	traits::{One, Zero},
};
use sp_std::{cmp::Ordering, collections::btree_map::BTreeMap, ops::Add, prelude::*};

use pallet_posts::Post;
use pallet_utils::{PostId, SpaceId};

use crate::RawEvent::PostGotInLottery;

// TODO: move all tests to df-integration-tests
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;
type NumberOfTipping = u32;

/// The pallet's configuration trait.
pub trait Config: system::Config + pallet_posts::Config {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	type Currency: Currency<Self::AccountId>;

	type TreasuryShare: Get<i16>;
	type PostAuthorsShare: Get<i16>;
	type TippersShare: Get<i16>;
	type NumberOfWinningTippers: Get<u32>;
	type NumberOfWinningPostAuthors: Get<u32>;
	type LotteryLength: Get<Self::BlockNumber>;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct LotterySpaceResults<T: Config> {
	winner_posts: Vec<PostId>,
	winning_posts_authors: BTreeMap<PostId, (T::AccountId, u64, BalanceOf<T>)>,
	winning_voters: Vec<(T::AccountId, u64, BalanceOf<T>)>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum LotteryStatusOfSpace<T: Config> {
	Done(LotterySpaceResults<T>),
	InProgress,
}

impl<T: Config> Default for LotteryStatusOfSpace<T> {
	fn default() -> Self {
		LotteryStatusOfSpace::InProgress
	}
}

impl<T: Config> LotteryStatusOfSpace<T> {
	pub fn is_done(&self) -> bool {
		return match self {
			LotteryStatusOfSpace::Done(_) => true,
			LotteryStatusOfSpace::InProgress => false,
		};
	}
}

type LotteryId<T> = <T as system::Config>::BlockNumber;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
struct Tip<T: Config> {
	tip_count: NumberOfTipping,
	tipper: T::AccountId,
	space_id: SpaceId,
	post_id: PostId,
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as LotteryModule {
        // Storage spaces of lotteries
        pub LotterySpacesIds:
        map hasher(blake2_128_concat) LotteryId<T> => Vec<SpaceId>;

        pub SpaceLotteryStatus:
        map hasher(blake2_128_concat) (LotteryId<T> , SpaceId) => LotteryStatusOfSpace<T>;

        // Counters
        pub PostLotteryVotes:
        map hasher(blake2_128_concat) (LotteryId<T> , PostId) => NumberOfTipping;

        pub UserLotteryVotes:
        map hasher(blake2_128_concat) (LotteryId<T> , T::AccountId) => NumberOfTipping;

        pub SpaceLotteryVotes:
        map hasher(blake2_128_concat) (LotteryId<T> , SpaceId) => NumberOfTipping;

        pub LotteryTips:
        map hasher(blake2_128_concat) LotteryId<T> => Vec<Tip<T>>
    }
}
// The pallet's events
decl_event!(
    pub enum Event<T>
    where
        BlockNumber = <T as system::Config>::BlockNumber,
    {
        /// Lottery ended event dispatched every X eras
        /// Every week for an era of 6Hours this is dispatched every 28 Era
        LotteryEnded(BlockNumber),
        /// For the first vote on a post on a new lottery this will be
        /// dispatched
        PostGotInLottery(BlockNumber, PostId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Config> {
    /// Insufficient balance
     InsufficientBalance,
    /// NoUpLottery
    NoUpLottery,
    /// Lottery Ended
    LotteryAlreadyEnded,
    CantVoteOnUnfoundPost,
    CantVoteOnUnspacedPost,
    }
}
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {


        // Initializing errors
        type Error = Error<T>;
        // Initializing events
        fn deposit_event() = default;

        #[weight = 0]
        pub fn vote_for_post(
        origin,
        post_id:PostId,
        tip_count:NumberOfTipping,
        ) -> DispatchResult {
			let tipper = ensure_signed(origin)?;
			let current_block_number = <system::Module<T>>::block_number();
			let voter_can_vote = Self::voter_has_enough_balance(&tipper ,tip_count)?;
            ensure!( voter_can_vote, Error::<T>::InsufficientBalance);
			let (lottery_exists ,current_lottery_id) = Self::lottery_exists(current_block_number);

			if !lottery_exists {
				Self::init_lottery(current_lottery_id);
			}

			let space_id = Self::incur_existence(post_id)?;
			let tip: Tip<T> = Tip {
				tip_count,
				tipper ,
				space_id ,
				post_id ,
			};

			Self::increase_counters(current_lottery_id , &tip);
			Self::commit_vote(current_lottery_id , &tip);

            Ok(())
        }


    fn on_finalize(n: T::BlockNumber) {


    }
}
}
impl<T: Config> Module<T> {
	fn commit_vote(
		lottery_id: LotteryId<T>,
		tip: &Tip<T>,
	) -> DispatchResult {
		if LotteryTips::<T>::contains_key(lottery_id) {
			LotteryTips::<T>::mutate(lottery_id, |c| c.push(tip.clone()))
		} else {
			LotteryTips::<T>::insert(lottery_id, vec![tip])
		}
		Ok(())
	}

	fn init_lottery(lottery_id: LotteryId<T>) {
		LotterySpacesIds::<T>::insert(lottery_id, Vec::<SpaceId>::new())
	}


	fn incur_existence(
		post_id: PostId
	) -> Result<SpaceId, sp_runtime::DispatchError> {
		let post: Option<Post<T>> = pallet_posts::Module::<T>::post_by_id(post_id);
		match post {
			Some(post) => {
				let space_id = post.space_id;
				match space_id {
					None => {
						Err(Error::<T>::CantVoteOnUnspacedPost.into())
					}
					Some(space_id) => {
						Ok(space_id)
					}
				}
			}
			_ => {
				Err(Error::<T>::CantVoteOnUnspacedPost.into())
			}
		}
	}

	fn increase_counters(
		lottery_id: LotteryId<T>,
		tip: &Tip<T>) -> DispatchResult {
		let lottery_post_key = (lottery_id, tip.post_id);
		let lottery_space_key = (lottery_id, tip.space_id);
		let space_id = tip.space_id;
		let lottery_voter_key = (lottery_id, tip.tipper.clone());
		let votes_count: NumberOfTipping = tip.tip_count;

		if PostLotteryVotes::<T>::contains_key(lottery_post_key) {
			PostLotteryVotes::<T>::mutate(
				lottery_post_key,
				|current| *current = *current + votes_count,
			);
		} else {
			PostLotteryVotes::<T>::insert(lottery_post_key, votes_count);
		}

		if SpaceLotteryVotes::<T>::contains_key(lottery_space_key) {
			PostLotteryVotes::<T>::mutate(
				lottery_space_key,
				|current| *current = *current + votes_count,
			);
		} else {
			PostLotteryVotes::<T>::insert(lottery_space_key, votes_count);
			// The in the DB since we do initialize the lottery and insert the Vector
			LotterySpacesIds::<T>::mutate(lottery_id, |c| c.push(space_id))
		}

		if UserLotteryVotes::<T>::contains_key(lottery_voter_key.clone()) {
			UserLotteryVotes::<T>::mutate(
				lottery_voter_key,
				|current| *current = *current + votes_count,
			);
		} else {
			UserLotteryVotes::<T>::insert(lottery_voter_key, votes_count);
		}
		Ok(())
	}


	fn voter_has_enough_balance(
		voter: &T::AccountId,
		number_of_votes: NumberOfTipping,
	) -> Result<bool, sp_runtime::DispatchError> {
		// Each vote costs 1 Native Unit
		let votes_cost: BalanceOf<T> = BalanceOf::<T>::from(number_of_votes);
		// Todo check if the balance don't include `ExistentialDeposit`
		let user_free_balance = <T as Config>::Currency::free_balance(voter);
		// Todo: include fees to the check
		Ok(user_free_balance >= votes_cost)
	}
	fn lottery_exists(block_number: T::BlockNumber) -> (bool, T::BlockNumber) {
		let mut lottery_id = block_number / T::LotteryLength::get();
		if lottery_id == Zero::zero() {
			lottery_id = One::one();
		}
		let exists = LotterySpacesIds::<T>::contains_key(lottery_id);
		log::info!(
			"Lottery init {:?} block number {:?} exists {:?}",
			lottery_id,
			block_number,
			exists
		);
		(exists, lottery_id)
	}

}
/*impl<T: Config> Module<T> {
    fn lottery_exists(block_number: T::BlockNumber) -> (bool, T::BlockNumber) {
        let mut lottery_id = block_number / T::BlockNumber::from(10 as u32);
        if lottery_id == Zero::zero() {
            lottery_id = One::one();
        }
        let exists = LotteryStatusByLotteryId::<T>::contains_key(lottery_id);
        log::info!(
            "Lottery init {:?} block number {:?} exists {:?}",
            lottery_id,
            block_number,
            exists
        );
        (exists, lottery_id)
    }

    fn init_lottery(lottery_id: T::BlockNumber) {
        LotteryStatusByLotteryId::<T>::insert(lottery_id, LotteryStatus::InProgress);
    }

    fn end_previous_lottery(current_lottery_id: T::BlockNumber) -> DispatchResult {
        let prev_lottery_id: T::BlockNumber = current_lottery_id - One::one();
        let lottery_exists = LotteryStatusByLotteryId::<T>::contains_key(prev_lottery_id);
        if !lottery_exists {
            return Ok(());
        }
        let prev_lottery: LotteryStatus<T> = LotteryStatusByLotteryId::<T>::get(prev_lottery_id);
        match prev_lottery {
            LotteryStatus::Done(_) => Ok(()),
            LotteryStatus::InProgress => {
                let lottery_results = Self::end_lottery(prev_lottery_id)?;
                LotteryStatusByLotteryId::<T>::insert(prev_lottery_id, LotteryStatus::Done(lottery_results));
                Ok(())
            }
        }
    }

    fn total_price() -> PostVotesNumber {
        return 10000;
    }

    fn end_lottery(lottery_id: T::BlockNumber) -> Result<LotteryResults<T>, sp_runtime::DispatchError> {
        let voter_price_share = Self::voter_price_share();
        let number_of_winning_posts = Self::number_of_winning_posts();
        let votes: Vec<(VoteKey<T>, PostVotesNumber)> = VotesForLottery::<T>::get(lottery_id);

        let mut user_total_number_of_votes: BTreeMap<T::AccountId, PostVotesNumber> = BTreeMap::new();
        let mut posts_votes_numbers: BTreeMap<PostId, PostVotesNumber> = BTreeMap::new();
        let mut total_votes: PostVotesNumber = 0;
        for ((_, voter, post_id, _nonce), vote) in votes.into_iter() {
            user_total_number_of_votes
                .entry(voter)
                .and_modify(|v| {
                    v.add(vote);
                })
                .or_insert(vote);

            posts_votes_numbers
                .entry(post_id)
                .and_modify(|v| {
                    v.add(vote);
                })
                .or_insert(vote);

            total_votes += vote;
        }
        let mut total_price = total_votes;
        // no votes
        if total_votes == 0 {
            log::info!("NO lottery votes for lottery {:?}", &lottery_id);
            let results = LotteryResults {
                spent: true,
                winner_posts: Vec::new(),
                winner_voter: None,
            };
            return Ok(results);
        }

        let mut users_probability_of_winning = user_total_number_of_votes.clone();
        users_probability_of_winning
            .values_mut()
            .for_each(|v| *v = *v / total_votes);

        let winner = users_probability_of_winning.into_iter().max_by_key(|(_, value)| *value);

        let mut winner_posts: Vec<_> = posts_votes_numbers.into_iter().collect();
        winner_posts.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let winner_posts_ids: Vec<_> = winner_posts
            .into_iter()
            .take(number_of_winning_posts as usize)
            .collect();
        let mut winner_ids: Vec<PostId> = vec![];
        let mut number_of_winners: PostVotesNumber = 0;
        for (post_id, _) in winner_posts_ids.iter() {
            winner_ids.push(*post_id);
            number_of_winners += 1;
        }

        /*	if (number_of_winners < number_of_winning_posts {
            // todo calculate new n from  `n = min(1,max(1,Math.floor(.5m)))`
            // todo use the new n to split the price across winner posts
        }*/
        match winner.clone() {
            None => {}
            Some((winner, _)) => {
                let mut voter_pierce = total_price * voter_price_share / 100;
                total_price = total_price * 100 - total_price * voter_price_share;
                total_price = total_price / 100;
                let voter_pierce = Self::u64_to_balance(voter_pierce)?;
                let current_balance_of_the_winner = <T as Config>::Currency::free_balance(&winner);
                <T as Config>::Currency::make_free_balance_be(&winner, current_balance_of_the_winner + voter_pierce);
            }
        }
        let post_price = total_price / number_of_winners;
        let post_price = Self::u64_to_balance(post_price)?;

        winner_ids.iter().for_each(|post_id| {
            let post: Option<Post<T>> = pallet_posts::Module::<T>::post_by_id(post_id);
            match post {
                None => {}
                Some(post) => {
                    let author = post.owner;
                    let current_balance_of_the_winner = <T as Config>::Currency::free_balance(&author);
                    <T as Config>::Currency::make_free_balance_be(&author, current_balance_of_the_winner + post_price);
                }
            }
        });
        let results = LotteryResults {
            spent: true,
            // todo include winning posts
            winner_posts: Vec::new(),
            winner_voter: winner.clone().map(|w| w.0),
        };
        log::info!("Lottery results {:?}", winner);

        return Ok(results);
    }

    fn commit_vote(vote_key: &VoteKey<T>, vote: PostVotesNumber, lottery_id: T::BlockNumber) {
        Votes::<T>::insert(vote_key.clone(), vote);
        if VotesForLottery::<T>::contains_key(lottery_id) {
            VotesForLottery::<T>::mutate(lottery_id, |v| v.push((vote_key.clone(), vote)))
        } else {
            VotesForLottery::<T>::insert(lottery_id, vec![(vote_key.clone(), vote)])
        }
    }

    fn voter_has_enough_balance(
        voter: &T::AccountId,
        number_of_votes: PostVotesNumber,
    ) -> Result<bool, sp_runtime::DispatchError> {
        // Each vote costs 1 Native Unit
        let votes_cost: BalanceOf<T> = Self::u64_to_balance(number_of_votes)?;
        // Todo check if the balance don't include `ExistentialDeposit`
        let user_free_balance = <T as Config>::Currency::free_balance(voter);
        // Todo: include fees to the check
        Ok(user_free_balance >= votes_cost)
    }

    fn u64_to_balance(n: u64) -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
        let balance: BalanceOf<T> = n.try_into().map_err(|_| "failed to convert u64 to balance ")?;
        Ok(balance)
    }
}*/
