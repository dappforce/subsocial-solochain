//! # Lottery Monetization Module
//!
//! The Monetization module by lottery method

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	ensure,
	traits::{Currency},
};
use frame_system::{self as system, ensure_signed};
use log;
use sp_runtime::{
	sp_std::convert::TryInto,
	traits::{One, Zero},
	RuntimeDebug,
};
use sp_std::{cmp::Ordering, collections::btree_map::BTreeMap, ops::Add, prelude::*};

use pallet_posts::Post;
use pallet_utils::PostId;

// TODO: move all tests to df-integration-tests
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_posts::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
}
type PostVotesNumber = u64;

pub type VoteKey<T: Trait> = (T::BlockNumber, T::AccountId, PostId, PostVotesNumber);

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct LotteryResults<T: Trait> {
	winner_voter: Option<T::AccountId>,
	winner_posts: Vec<(PostId, PostVotesNumber)>,
	spent: bool,
}


#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum LotteryStatus<T: Trait> {
	Done(LotteryResults<T>),
	InProgress,
}

impl<T: Trait> Default for LotteryStatus<T> {
	fn default() -> Self {
		LotteryStatus::InProgress
	}
}
impl<T: Trait> LotteryStatus<T> {
	pub fn is_done(&self) -> bool {
		return match self {
			LotteryStatus::Done(_) => true,
			LotteryStatus::InProgress => false,
		};
	}
}
pub const VOTER_SHARE: u64 = 20;
pub const NUMBER_OF_WINNING_POSTS: u64 = 3;

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as LotteryModule {
	 /// An id for the next donation.

	pub VoterPricePercentage get(fn voter_price_share): u64 = VOTER_SHARE;
	pub NumberOfWinningPosts get(fn number_of_winning_posts): u64 = NUMBER_OF_WINNING_POSTS;

	pub LotteryStatusByLotteryId get(fn lottery_status_by_Lottery_id):
	map hasher(blake2_128_concat) T::BlockNumber => LotteryStatus<T>;

	pub PostVotesNumberOfLottery get(fn post_votes_number_of_lottery):
	map hasher(blake2_128_concat) (T::BlockNumber , PostId ) => PostVotesNumber;

	pub Votes get(fn votes):
	map hasher(blake2_128_concat) VoteKey<T> => Option<PostVotesNumber>;

	pub VotesForLottery get (fn votes_for_lottery):
	map hasher(blake2_128_concat) T::BlockNumber => Vec<(VoteKey<T> ,PostVotesNumber)>;

	}
}
// The pallet's events
decl_event!(
	pub enum Event<T>
	where
		BlockNumber = <T as system::Trait>::BlockNumber,
		VoteKey = VoteKey<T>,
	{
		/// Lottery ended event dispatched every X eras
		/// Every week for an era of 6Hours this is dispatched every 28 Era
		LotteryEnded(BlockNumber),
		/// For the first vote on a post on a new lottery this will be
		/// dispatched
		PostGotInLottery(BlockNumber, PostId),
		// ..
		UserVotted(VoteKey, PostVotesNumber),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
	/// Insufficient balance
	 InsufficientBalance,
	/// NoUpLottery
	NoUpLottery,
	/// Lottery Ended
	LotteryAlreadyEnded,
	}
}
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {


		// Initializing errors
		type Error = Error<T>;
		// Initializing events
		fn deposit_event() = default;

		#[weight = 0]
		pub fn vote_for_post(
		origin,
		post_id:PostId,
		post_votes: PostVotesNumber,
		) -> DispatchResult {
			let voter = ensure_signed(origin)?;
			let current_block_number = <system::Module<T>>::block_number();
			// check the user balance if they can buy votes
			let voter_can_vote = Self::voter_has_enough_balance(&voter,post_votes)?;
			ensure!( voter_can_vote, Error::<T>::InsufficientBalance);
			// Check the lottery existance
			let (lottery_exists ,current_lottery_id) = Self::lottery_exists(current_block_number);
			// Create a lottery if there isn't one up for the current period
			if  !lottery_exists {
				Self::init_lottery(current_lottery_id)
			}
			let post_lottery_key :(T::BlockNumber , PostId )= (current_lottery_id , post_id);
			// Number of post`s votes
			let post_votes_num = PostVotesNumberOfLottery::<T>::get(post_lottery_key);
			// User vote key
			let user_vote_key:VoteKey<T> = (current_lottery_id, voter.clone(), post_id ,post_votes_num);
			// Commit user vote to storage
			Self::commit_vote(&user_vote_key , post_votes , current_lottery_id );
			let balance_to_reduce:BalanceOf<T> = post_votes.try_into().map_err(|_| "failed to ")?;
			let user_free_balance = <T as Trait >::Currency::free_balance(&voter);
			<T as Trait >::Currency::make_free_balance_be( &voter , user_free_balance - balance_to_reduce);
			// Increase post's number of votes
			if post_votes_num == 0 {
				PostVotesNumberOfLottery::<T>::insert(post_lottery_key , post_votes);
				Self::deposit_event(Event::<T>::PostGotInLottery(current_lottery_id ,post_id));
			} else {
				 PostVotesNumberOfLottery::<T>::mutate(post_lottery_key , |votes| *votes += post_votes);
			}

			Self::deposit_event(Event::<T>::UserVotted( user_vote_key, post_votes));

			Ok(())
		}


	fn on_finalize(n: T::BlockNumber) {
		let (lottery_exists ,lottery_id) = Self::lottery_exists(n);
		if !lottery_exists {
			Self::init_lottery(lottery_id);
			Self::end_previous_lottery(lottery_id);
			}
		}

	}
}
impl<T: Trait> Module<T> {
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
				let mut	voter_pierce = total_price * voter_price_share / 100;
				total_price = total_price * 100 - total_price * voter_price_share;
				total_price = total_price / 100;
				let voter_pierce = Self::u64_to_balance(voter_pierce)?;
				let current_balance_of_the_winner = <T as Trait>::Currency::free_balance(&winner);
				<T as Trait>::Currency::make_free_balance_be(&winner, current_balance_of_the_winner + voter_pierce);
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
					let current_balance_of_the_winner = <T as Trait>::Currency::free_balance(&author);
					<T as Trait>::Currency::make_free_balance_be(&author, current_balance_of_the_winner + post_price);
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
		let user_free_balance = <T as Trait>::Currency::free_balance(voter);
		// Todo: include fees to the check
		Ok(user_free_balance >= votes_cost)
	}

	fn u64_to_balance(n: u64) -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
		let balance: BalanceOf<T> = n.try_into().map_err(|_| "failed to convert u64 to balance ")?;
		Ok(balance)
	}
}
