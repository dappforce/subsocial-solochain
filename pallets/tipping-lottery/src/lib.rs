//! # Lottery Monetization Module
//!
//! The Monetization module by lottery method

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Currency,
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
use pallet_utils::{PostId, SpaceId, WhoAndWhen};

use crate::RawEvent::PostGotInLottery;

// TODO: move all tests to df-integration-tests
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;
type NumberOfTipping = u32;

pub type LotteryId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub struct When<T: Config> {
    pub block: T::BlockNumber,
    pub time: T::Moment,
}

impl<T: Config> Default for When<T> {
    fn default() -> Self {
        When {
            block: <system::Module<T>>::block_number(),
            time: <pallet_timestamp::Module<T>>::now(),
        }
    }
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum LotteryDuration<T: Config> {
    Day1(T::BlockNumber),
    Days8(T::BlockNumber),
    Days28(T::BlockNumber),
    Range(T::BlockNumber, T::BlockNumber),
}

impl<T: Config> Default for LotteryDuration<T> {
    fn default() -> Self {
        let current_block = <system::Module<T>>::block_number();
        Self::Day1(current_block + One::one())
    }
}

impl<T: Config> LotteryDuration<T> {
    fn current_block() -> T::BlockNumber {
        <system::Module<T>>::block_number()
    }

    fn is_valid_start(block_number: T::BlockNumber) -> bool {
        return Self::current_block() <= block_number;
    }

    fn is_valid_range(from: T::BlockNumber, to: T::BlockNumber) -> bool {
        // todo fix the threshold
        return Self::current_block() <= from && from + T::BlockNumber::from(100 as u32) < to;
    }

    fn is_present(&self, n: T::BlockNumber, threshold: Option<T::BlockNumber>) -> bool {
        let time_per_block: u8 = 6000;
        let (starts_from ,ends_at) = match *self {
            LotteryDuration::Day1(starts_from) => {
                let day_block_count: u8 = 86_400_000 / time_per_block;
                (
                    starts_from,
                    starts_from + T::BlockNumber::from(day_block_count),
                )
            }
            LotteryDuration::Days8(starts_from) => {
                let day8_block_count: u8 = 86_400_000 * 8 / time_per_block;
                (
                    starts_from,
                    starts_from + T::BlockNumber::from(day8_block_count),
                )
            }
            LotteryDuration::Days28(starts_from) => {
                let day28_block_count: u8 = 86_400_000 * 24 / time_per_block;
                (
                    starts_from,
                    starts_from + T::BlockNumber::from(day28_block_count),
                )
            }
            LotteryDuration::Range(starts_from, ends_at) => (starts_from, ends_at),
        };
        match threshold {
            None => starts_from <= n && n <= ends_at,
            Some(threshold) => starts_from - threshold <= n && n <= ends_at + threshold,
        }
    }
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub struct LotteryRewardingConfig {
    pub treasury_share: u16,
    pub post_authors_share: u16,
    pub tippers_share: u16,
    pub number_of_winning_tippers: u16,
    pub number_of_winning_post_authors: u16,
}

impl Default for LotteryRewardingConfig {
    fn default() -> Self {
        Self {
            treasury_share: 20,
            post_authors_share: 40,
            tippers_share: 40,
            number_of_winning_tippers: 5,
            number_of_winning_post_authors: 5,
        }
    }
}

pub const FIRST_POST_ID: LotteryId = 1;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub struct Lottery<T: Config> {
    pub id: LotteryId,
    pub space_id: SpaceId,
    pub duration: LotteryDuration<T>,
    pub rewarding_config: LotteryRewardingConfig,

    pub created: When<T>,
    pub updated: Option<When<T>>,

    pub is_canceled: bool,
    pub canceled: Option<When<T>>,
    pub is_done: bool,
    pub done: Option<When<T>>,
}

impl<T: Config> Lottery<T> {
    fn new(
        id: LotteryId,
        space_id: SpaceId,
        rewarding_config: LotteryRewardingConfig,
        duration: LotteryDuration<T>,
    ) -> Self {
        Lottery {
            id,
            space_id,
            duration,
            rewarding_config,
            created: Default::default(),
            updated: None,
            is_canceled: false,
            canceled: None,
            is_done: false,
            done: None,
        }
    }

    fn from_id(id: LotteryId) -> Option<Self> {
        LotteryById::<T>::get(id)
    }

    fn can_be_ended(&self, n: T::BlockNumber) -> bool {
        !self.is_done && !self.duration.is_present(n,None)
    }

    fn is_open_for_voting(&self) -> bool {
        let current_block = <system::Module<T>>::block_number();
        self.can_vote_at_block(current_block)
    }

    fn can_vote_at_block(&self, n: T::BlockNumber) -> bool {
        let status = !self.is_canceled && !self.is_done && !self.is_canceled;
        status && self.duration.is_present(n, None)
    }

    fn is_started(&self, n: T::BlockNumber) -> bool {
        self.duration.is_present(n, None)
    }
}

/// The pallet's configuration trait.
pub trait Config: system::Config + pallet_posts::Config + pallet_spaces::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    type Currency: Currency<Self::AccountId>;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct LotteryResults<T: Config> {
    winner_posts: Vec<PostId>,
    winning_posts_authors: BTreeMap<PostId, (T::AccountId, u64, BalanceOf<T>)>,
    winning_voters: Vec<(T::AccountId, u64, BalanceOf<T>)>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum LotteryStatusOfSpace<T: Config> {
    Done(LotteryResults<T>),
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

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
struct Tip<T: Config> {
    tip_count: NumberOfTipping,
    tipper: T::AccountId,
    space_id: SpaceId,
    post_id: PostId,
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as TippingLotteryModule {
        // counter for lottery ids
        pub NextLotteryId get(fn next_lottery_id) : LotteryId = FIRST_POST_ID;

        // Store
        pub SpacesWithNoDoneLotteries get(fn spaces_with_not_done_lotteries) : Vec<SpaceId>;

        // Store lotteries by id
        pub LotteryById get(fn lottery_by_id):
        map hasher(blake2_128_concat) LotteryId => Option<Lottery<T>>;

        // Lotteries that are not done yet
        // This will contain lotteries  present with a threshold of 100 blocks before start
        pub InProgressLottires get(fn in_progress_lottaries):
        map hasher(blake2_128_concat)  SpaceId => Vec<LotteryId>;

        // Lotteries that are done and the rewards are spent
        pub SpaceDoneLotteries get(fn space_done_lotteries) :
        map hasher(blake2_128_concat) SpaceId => Vec<LotteryId>;

        // Lotteries that are not yet near to be in progress
        pub SpaceWaitingLotteries get(fn space_waiting_lotteries) :
        map hasher(blake2_128_concat) SpaceId => Vec<LotteryId>;

        // Full list of lotteries for a given space
        pub LotteryIdsOfSpace get(fn lottery_ids_of_space):
        map hasher(blake2_128_concat) SpaceId => Vec<LotteryId>;

        pub SpaceLotteryStatus:
        map hasher(blake2_128_concat)  LotteryId => LotteryStatusOfSpace<T>;

        // Counters
        pub PostLotteryVotes:
        map hasher(blake2_128_concat) (LotteryId , PostId) => NumberOfTipping;

        pub UserLotteryVotes:
        map hasher(blake2_128_concat) (LotteryId , T::AccountId) => NumberOfTipping;

        pub SpaceLotteryVotes:
        map hasher(blake2_128_concat) (LotteryId, SpaceId) => NumberOfTipping;

        pub LotteryTips:
        map hasher(blake2_128_concat) LotteryId => Vec<Tip<T>>
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
 	LotteryIsDone,
    LotteryNotInProgress,
    /// Lottery Ended
    LotteryAlreadyEnded,
    CantVoteOnUnfoundPost,
    CantVoteOnUnspacedPost,
    PostNotFound,

    // lottery
    LotteryNotFound
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
        pub fn create_lottery(
        origin,
        space_id: SpaceId,
        rewarding_config: LotteryRewardingConfig,
        duration: LotteryDuration<T>,
        ) -> DispatchResult {
            let maybe_space_owner = ensure_signed(origin)?;
            let space = <pallet_spaces::Module::<T>>::require_space(space_id)?;
            space.ensure_space_owner(maybe_space_owner)?;
			let current_block_number = <system::Module<T>>::block_number();
            let next_lottery_id = Self::next_lottery_id();
            let lottery = Lottery::<T>::new(next_lottery_id ,space_id , rewarding_config ,duration);
            LotteryById::<T>::insert(next_lottery_id,lottery);
            Self::commit_lottery(&lottery,current_block_number);
            Ok(())
        }

        #[weight = 0]
        pub fn vote_for_post(
        origin,
        post_id:PostId,
        tip_count:NumberOfTipping,
        lottery_id:LotteryId
        ) -> DispatchResult {
            let tipper = ensure_signed(origin)?;
            let current_block_number = <system::Module<T>>::block_number();
            // check if the user have enough balance for the votes
            let voter_can_vote = Self::voter_has_enough_balance(&tipper ,tip_count)?;
            ensure!( voter_can_vote, Error::<T>::InsufficientBalance);
            let lottery:Lottery<T> = Lottery::<T>::from_id(lottery_id)
            .ok_or(Error::<T>::LotteryNotFound)?;

            ensure!( !lottery.is_done, Error::<T>::LotteryIsDone);
            ensure!( lottery.can_vote_at_block(current_block_number), Error::<T>::LotteryNotInProgress);

            let tip: Tip<T> = Tip {
                tip_count,
                tipper ,
                space_id:lottery.space_id ,
                post_id ,
            };

            Self::increase_counters(lottery.id , &tip);
            Self::commit_vote(lottery.id , &tip);

            Ok(())
        }


    fn on_finalize(n: T::BlockNumber) {


    }
}
}
impl<T: Config> Module<T> {
    fn inc_lottery_id() {
		<NextLotteryId>::mutate(|n| {
            *n += 1;
        });
    }

    fn commit_lottery(lottery: &Lottery<T>, n: T::BlockNumber) {
        // check if the lottery is started or near from string
        let is_near_from_starting = lottery
            .duration
            .is_present(n, Some(T::BlockNumber::from(100 as u32)));
        if is_near_from_starting {
            // Insert the lottery on the starting queue
            if <InProgressLottires>::contains_key(lottery.space_id) {
                <InProgressLottires>::mutate(lottery.space_id, |in_progress| {
                    in_progress.push(lottery.id)
                });
            }
            <InProgressLottires>::insert(lottery.space_id, vec![lottery.id]);
        } else {
            // Insert the lottery on the waiting queue
            if <SpaceWaitingLotteries>::contains_key(lottery.space_id) {
                <SpaceWaitingLotteries>::mutate(lottery.space_id, |in_progress| {
                    in_progress.push(lottery.id)
                });
            }
            <SpaceWaitingLotteries>::insert(lottery.space_id, vec![lottery.id]);
        }

        // insert the lottery for the full list of lotteries
        if <LotteryIdsOfSpace>::contains_key(lottery.space_id) {
            <LotteryIdsOfSpace>::mutate(lottery.space_id, |in_progress| {
                in_progress.push(lottery.id)
            });
        }
        <LotteryIdsOfSpace>::insert(lottery.space_id, vec![lottery.id]);
        // ad the space id to the list of
        <SpacesWithNoDoneLotteries>::mutate(|space_ids| space_ids.push(lottery.space_id));
        // increase lottery count
        Self::inc_lottery_id();
        // insert the lottery
        LotteryById::<T>::insert(lottery.id, lottery);
    }

    fn commit_vote(lottery_id: LotteryId, tip: &Tip<T>) -> DispatchResult {
        if LotteryTips::<T>::contains_key(lottery_id) {
            LotteryTips::<T>::mutate(lottery_id, |c| c.push(tip.clone()))
        } else {
            LotteryTips::<T>::insert(lottery_id, vec![tip])
        }
        Ok(())
    }

    fn incur_existence(post_id: PostId) -> DispatchResult {
        let post: Option<Post<T>> = pallet_posts::Module::<T>::post_by_id(post_id);
        post.map(|p| p.space_id.map(|_| ()).ok_or(Error::<T>::PostNotFound.into()))
            .ok_or::<Error<T>>(Error::<T>::CantVoteOnUnspacedPost.into())?
    }

    fn increase_counters(lottery_id: LotteryId, tip: &Tip<T>) -> DispatchResult {
        let lottery_post_key = (lottery_id, tip.post_id);
        let lottery_space_key = (lottery_id, tip.space_id);
        let space_id = tip.space_id;
        let lottery_voter_key = (lottery_id, tip.tipper.clone());
        let votes_count: NumberOfTipping = tip.tip_count;
		// increase or set number of votes on a post in scope of the lottery
        if <PostLotteryVotes>::contains_key(lottery_post_key) {
            <PostLotteryVotes>::mutate(lottery_post_key, |current| {
                *current = *current + votes_count
            });
        } else {
            <PostLotteryVotes>::insert(lottery_post_key, votes_count);
        }
		// increase or set number of votes on a space in scope of the lottery
        if <SpaceLotteryVotes>::contains_key(lottery_space_key) {
			<SpaceLotteryVotes>::mutate(lottery_space_key, |current| {
                *current = *current + votes_count
            });
        } else {
			<SpaceLotteryVotes>::insert(lottery_space_key, votes_count);
        }

        if <UserLotteryVotes<T>>::contains_key(lottery_voter_key.clone()) {
			<UserLotteryVotes<T>>::mutate(lottery_voter_key, |current| {
                *current = *current + votes_count
            });
        } else {
			<UserLotteryVotes<T>>::insert(lottery_voter_key, votes_count);
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



    fn end_lottery(lottery_id: LotteryId) {
		unimplemented!()
    }

    fn get_lottery_winner_accounts(lottery_id: LotteryId) {
        unimplemented!()
    }

    fn end_space_lottery(space_id: SpaceId, lottery_id: LotteryId) {}
}
