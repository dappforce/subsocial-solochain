use crate::Balance;

pub const UNITS: Balance = 100_000_000_000;
pub const DOLLARS: Balance = UNITS;            // 100_000_000_000
pub const CENTS: Balance = DOLLARS / 100;      // 1_000_000_000
pub const MILLICENTS: Balance = CENTS / 1_000; // 1_000_000

pub const fn deposit(items: u32, bytes: u32) -> Balance {
    items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
}