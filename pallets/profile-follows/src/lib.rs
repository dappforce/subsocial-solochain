#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    storage::StorageMap as OldStorageMap,
};
use sp_std::prelude::*;

use pallet_profiles::{Module as Profiles, SocialAccountById};
use pallet_utils::remove_from_vec;

pub mod rpc;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
    };
    use frame_system::{ensure_signed, pallet_prelude::*};
    use super::*;

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config
    + pallet_utils::Config
    + pallet_profiles::Config
    {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type BeforeAccountFollowed: BeforeAccountFollowed<Self>;

        type BeforeAccountUnfollowed: BeforeAccountUnfollowed<Self>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T>{
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(4, 4))]
        pub fn follow_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResultWithPostInfo {
            let follower = ensure_signed(origin)?;

            ensure!(follower != account, Error::<T>::AccountCannotFollowItself);
            ensure!(!<AccountFollowedByAccount<T>>::contains_key((follower.clone(), account.clone())),
                Error::<T>::AlreadyAccountFollower);

            let mut follower_account = Profiles::get_or_new_social_account(follower.clone());
            let mut followed_account = Profiles::get_or_new_social_account(account.clone());

            follower_account.inc_following_accounts();
            followed_account.inc_followers();

            T::BeforeAccountFollowed::before_account_followed(
                follower.clone(), follower_account.reputation, account.clone())?;

            <SocialAccountById<T>>::insert(follower.clone(), follower_account);
            <SocialAccountById<T>>::insert(account.clone(), followed_account);
            <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(account.clone()));
            <AccountFollowers<T>>::mutate(account.clone(), |ids| ids.push(follower.clone()));
            <AccountFollowedByAccount<T>>::insert((follower.clone(), account.clone()), true);

            Self::deposit_event(Event::AccountFollowed(follower, account));
            Ok(Default::default())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(4, 4))]
        pub fn unfollow_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResultWithPostInfo {
            let follower = ensure_signed(origin)?;

            ensure!(follower != account, Error::<T>::AccountCannotUnfollowItself);
            ensure!(<AccountFollowedByAccount<T>>::contains_key((follower.clone(), account.clone())), Error::<T>::NotAccountFollower);

            let mut follower_account = Profiles::social_account_by_id(follower.clone()).ok_or(Error::<T>::FollowerAccountNotFound)?;
            let mut followed_account = Profiles::social_account_by_id(account.clone()).ok_or(Error::<T>::FollowedAccountNotFound)?;

            follower_account.dec_following_accounts();
            followed_account.dec_followers();

            T::BeforeAccountUnfollowed::before_account_unfollowed(follower.clone(), account.clone())?;

            <SocialAccountById<T>>::insert(follower.clone(), follower_account);
            <SocialAccountById<T>>::insert(account.clone(), followed_account);
            <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |account_ids| remove_from_vec(account_ids, account.clone()));
            <AccountFollowers<T>>::mutate(account.clone(), |account_ids| remove_from_vec(account_ids, follower.clone()));
            <AccountFollowedByAccount<T>>::remove((follower.clone(), account.clone()));

            Self::deposit_event(Event::AccountUnfollowed(follower, account));
            Ok(Default::default())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        AccountFollowed(/* follower */ T::AccountId, /* following */ T::AccountId),
        AccountUnfollowed(/* follower */ T::AccountId, /* unfollowing */ T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Follower social account was not found by id.
        FollowerAccountNotFound,
        /// Social account that is being followed was not found by id.
        FollowedAccountNotFound,

        /// Account can not follow itself.
        AccountCannotFollowItself,
        /// Account can not unfollow itself.
        AccountCannotUnfollowItself,

        /// Account (Alice) is already a follower of another account (Bob).
        AlreadyAccountFollower,
        /// Account (Alice) is not a follower of another account (Bob).
        NotAccountFollower,
    }

    #[pallet::storage]
    #[pallet::getter(fn account_followers)]
    pub type AccountFollowers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_followed_by_account)]
    pub type AccountFollowedByAccount<T: Config> = StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn accounts_followed_by_account)]
    pub type AccountsFollowedByAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;




    /// Handler that will be called right before the account is followed.
    pub trait BeforeAccountFollowed<T: Config> {
        fn before_account_followed(follower: T::AccountId, follower_reputation: u32, following: T::AccountId) -> DispatchResult;
    }

    impl<T: Config> BeforeAccountFollowed<T> for () {
        fn before_account_followed(_follower: T::AccountId, _follower_reputation: u32, _following: T::AccountId) -> DispatchResult {
            Ok(())
        }
    }

    /// Handler that will be called right before the account is unfollowed.
    pub trait BeforeAccountUnfollowed<T: Config> {
        fn before_account_unfollowed(follower: T::AccountId, following: T::AccountId) -> DispatchResult;
    }

    impl<T: Config> BeforeAccountUnfollowed<T> for () {
        fn before_account_unfollowed(_follower: T::AccountId, _following: T::AccountId) -> DispatchResult {
            Ok(())
        }
    }

}