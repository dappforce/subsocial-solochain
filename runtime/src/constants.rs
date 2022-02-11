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
	pub const BLOCK: BlockNumber = 1;
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
	pub const WEEKS: BlockNumber = DAYS * 7;
	pub const MONTHS: BlockNumber = DAYS * 30;
}

pub mod free_calls {
    use pallet_free_calls::{QuotaToWindowRatio, WindowConfig};
    use crate::BlockNumber;
    use super::time::*;

	pub const FREE_CALLS_PER_SUB: u16 = 10;

	/// Make sure that every next period is equal or smaller and ratio is equal or bigger.
    pub const FREE_CALLS_WINDOWS_CONFIG: [WindowConfig<BlockNumber>; 3] = [
        /// Window that lasts a day and has 100% of the allocated quota.
		WindowConfig::new(1 * DAYS, QuotaToWindowRatio::new(1)),
        /// Window that lasts an hour and has (1/3) of the allocated quota.
        WindowConfig::new(1 * HOURS, QuotaToWindowRatio::new(3)),
        /// Window that lasts for 5 minutes and has (1/10) of the allocated quota.
        WindowConfig::new(5 * MINUTES, QuotaToWindowRatio::new(10)),
    ];
}
