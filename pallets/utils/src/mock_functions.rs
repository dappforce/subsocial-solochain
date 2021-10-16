use super::*;

// Valid CID of the empty file
const VALID_IPFS_CID: &[u8; IPFS_CID_V0_LENGTH] = b"QmbFMke1KXqnYyBBWxB74N4c5SBnJMVAiMNRcGu6x1AwQH";

// CID of the file with the text: "update"
const UPDATED_IPFS_CID: &[u8; IPFS_CID_V0_LENGTH] = b"QmZ3EnvnMrFJ7R5JZDMDBxsSvePeHTciykmgHwGc3aeRnu";

#[cfg(feature = "frame-benchmarking")]
pub mod bench {
    use super::{BalanceOf, Config, Currency};
    use frame_benchmarking::whitelisted_caller;
    use sp_runtime::traits::Bounded;

    pub fn caller_with_balance<T: Config>() -> T::AccountId {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        caller
    }
}

#[cfg(feature = "std")]
pub mod ext_builder {
    use sp_runtime::traits::One;
    use sp_std::marker::PhantomData;
    use sp_io::TestExternalities;

    pub struct DefaultExtBuilder<TestRuntime: frame_system::Config>(PhantomData<TestRuntime>);

    impl<TestRuntime: frame_system::Config> DefaultExtBuilder<TestRuntime> {
        pub fn build() -> TestExternalities {
            let storage = frame_system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();

            let mut ext = TestExternalities::from(storage);
            ext.execute_with(|| frame_system::Pallet::<TestRuntime>::set_block_number(One::one()));

            ext
        }
    }
}

/// Returns valid IPFS CID of the empty file
pub fn valid_content_ipfs() -> Content {
    Content::IPFS(VALID_IPFS_CID.to_vec())
}

/// Returns invalid IPFS CID
pub fn invalid_content_ipfs() -> Content {
    Content::IPFS(b"QmbFMke1KXqnYy".to_vec())
}

/// Returns valid IPFS CID that differs from the empty file CID.
pub fn updated_content_ipfs() -> Content {
    Content::IPFS(UPDATED_IPFS_CID.to_vec())
}

/// Returns valid handle of the `MaxHandleLength` filled with "a" letters.
pub fn valid_max_length_handle<T: Config>() -> Vec<u8> {
    vec![b'a'; T::MaxHandleLen::get() as usize]
}

/// Returns valid handle of the `MaxHandleLength` filled with "b" letters.
pub fn updated_max_length_handle<T: Config>() -> Vec<u8> {
    vec![b'b'; T::MaxHandleLen::get() as usize]
}

/// Returns valid handle of the `MinHandleLength` filled with "c" letters.
pub fn valid_min_length_handle<T: Config>() -> Vec<u8> {
    vec![b'c'; T::MinHandleLen::get() as usize]
}

/// Returns valid handle of the `MinHandleLength` filled with "d" letters.
pub fn updated_min_length_handle<T: Config>() -> Vec<u8> {
    vec![b'd'; T::MaxHandleLen::get() as usize]
}
