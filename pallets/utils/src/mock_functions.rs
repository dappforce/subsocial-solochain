use super::*;
use sp_io::TestExternalities;
use sp_runtime::traits::{Bounded, One};
use sp_std::marker::PhantomData;

pub struct DefaultExtBuilder<TestRuntime: system::Config>(PhantomData<TestRuntime>);

impl<TestRuntime: system::Config> DefaultExtBuilder<TestRuntime> {
    pub fn build() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| system::Pallet::<TestRuntime>::set_block_number(One::one()));

        ext
    }
}

pub fn valid_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCaidP37UdDnjFY5aQuiBrbqdyoW1CaDgwxkD4".to_vec())
}

pub fn invalid_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6DaazhR8".to_vec())
}

