pub mod currency {
	use subsocial_primitives::Balance;

	pub const UNITS: Balance = 100_000_000_000;
	pub const DOLLARS: Balance = UNITS;            // 100_000_000_000
	pub const CENTS: Balance = DOLLARS / 100;      // 1_000_000_000
	pub const MILLICENTS: Balance = CENTS / 1_000; // 1_000_000

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}

pub mod time {
	use subsocial_primitives::{Moment, BlockNumber};

	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

pub mod rate_limiter {
	use pallet_rate_limiter::RateConfig;
	use crate::BlockNumber;
	use super::time::*;

	pub const RATE_CONFIGS: [RateConfig<BlockNumber>; 1] = [
		/*RateConfig {
			period: 5 * MINUTES,
			max_permits: 10
		},
		RateConfig {
			period: 1 * HOURS,
			max_permits: 20
		},*/
		RateConfig {
			period: 1 * DAYS,
			max_permits: 40
		}
	];
}
