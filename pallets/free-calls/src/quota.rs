use sp_std::cmp::max;
use sp_std::num::NonZeroU16;
use static_assertions::const_assert;

/// Type to keep track of how many calls in the quota were used in a particular window.
pub type NumberOfCalls = u16;

/// The maximum number of free calls allocated for the consumer for the biggest window.
///
/// ## Example:
/// Window 1 => 10 Hours  <-- The biggest window
/// Window 2 => 1 Hour
/// Window 3 => 5 Minutes
pub type MaxQuota = NumberOfCalls;

/// Fraction of the [MaxQuota].
///
/// The [FractionOfMaxQuota] is a non-zero integer that represents the numerator when dividing by
/// the [MAX_QUOTA_DECIMALS].
///
/// You can use [max_quota_percentage] macro for convince.
///
/// ## Example:
/// - Assuming that [MAX_QUOTA_DECIMALS] is 10
///     - 1 fraction of max quota is 10%
///     - 5 fraction of max quota is 50%
/// - Assuming that [MAX_QUOTA_DECIMALS] is 1000
///     - 1 fraction of max quota is 0.1%
///     - 10 fraction of max quota is 1%
pub type FractionOfMaxQuota = NonZeroU16;

/// The number used to evaluate the [FractionOfMaxQuota].
///
/// Must be non-zero
pub const MAX_QUOTA_DECIMALS: u16 = 10_000;
const_assert!(MAX_QUOTA_DECIMALS != 0);

/// Evaluating the fraction of max quota based on the [MAX_QUOTA_DECIMALS].
///
/// The minuteman value that will be returned from the function is 1 unless [max_quota] is zero,
/// then the result is zero.
///
/// ## Example
/// Max quota is 10.
/// Fraction is equal to 10%.
/// Result will be 1.
///
/// Max quota is 10.
/// Fraction is equal to 1%.
/// Result will still be 1, since this is the mimuman value it can get.
pub(crate) fn evaluate_quota(max_quota: MaxQuota, fraction: FractionOfMaxQuota) -> NumberOfCalls {
    if max_quota == 0 {
        return 0;
    }
    // we need to cast to u64 to avoid overflowing.
    max(1, (max_quota as u64 * fraction.get() as u64 / MAX_QUOTA_DECIMALS as u64) as NumberOfCalls)
}

/// A convenience macro used to convert a floating number representing a percentage to non-zero
/// integer of type [FractionOfMaxQuota].
#[macro_export]
macro_rules! max_quota_percentage {
    ($percentage:expr) => {{
        $crate::__validate_percentage!($percentage);
        let fraction =
            ($crate::quota::MAX_QUOTA_DECIMALS as f32 * ($percentage as f32) / 100f32) as u16;
        match $crate::quota::FractionOfMaxQuota::new(fraction) {
            Some(non_zero) => non_zero,
            None => panic!("quota_fraction must be non zero"),
        }
    }};
}

/// Ensures that [percentage] is a constant and not larger than 100.0.
#[cfg(not(test))]
#[macro_export]
macro_rules! __validate_percentage {
    ($percentage:expr) => {{
        static_assertions::const_assert!($percentage as f32 <= 100f32);
    }};
}


/// (Only for tests) Only check that percentage is not larger than 100%.
#[cfg(test)]
#[macro_export]
macro_rules! __validate_percentage {
    ($percentage:expr) => {{
        if $percentage as f32 > 100f32 {
            panic!("percentage should be less than or equal to 100%");
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::quota::{evaluate_quota, MaxQuota, NumberOfCalls, MAX_QUOTA_DECIMALS};
    use rstest::rstest;

    #[rstest]
    #[case(100.0, 1.0)]
    #[case(1.0, 0.01)]
    #[case(33.33, 0.3333)]
    #[case(2.5, 0.025)]
    #[case(25.15, 0.2515)]
    fn max_quota_percentage_should_work_as_expected(
        #[case] percentage: f32,
        #[case] multiplier: f32,
    ) {
        assert_eq!(
            max_quota_percentage!(percentage).get(),
            (MAX_QUOTA_DECIMALS as f32 * multiplier) as u16,
            "max_quota_percentage {}%",
            percentage,
        );
    }

    #[rstest]
    #[case(0, 1.0, 0)]
    #[case(0, 10.0, 0)]
    #[case(0, 33.33, 0)]
    #[case(0, 100.0, 0)]
    //////////////////
    #[case(1, 1.0, 1)]
    #[case(1, 10.0, 1)]
    #[case(1, 33.33, 1)]
    #[case(1, 100.0, 1)]
    //////////////////
    #[case(10, 1.0, 1)]
    #[case(10, 10.0, 1)]
    #[case(10, 33.33, 3)]
    #[case(10, 55.5, 5)]
    #[case(10, 100.0, 10)]
    //////////////////
    #[case(320, 1.0, 3)]
    #[case(320, 10.0, 32)]
    #[case(320, 33.0, 105)]
    #[case(320, 33.33, 106)]
    #[case(320, 55.5, 177)]
    #[case(320, 100.0, 320)]
    //////////////////
    #[case(10_000, 0.01, 1)]
    #[case(10_000, 0.125, 12)]
    #[case(10_000, 1.0, 100)]
    #[case(10_000, 2.345, 234)]
    #[case(10_000, 10.0, 1_000)]
    #[case(10_000, 33.0, 3_300)]
    #[case(10_000, 33.33, 3_333)]
    #[case(10_000, 55.5, 5_550)]
    #[case(10_000, 100.0, 10_000)]
    fn evaluate_quota_should_work_as_expected(
        #[case] max_quota: MaxQuota,
        #[case] percentage: f32,
        #[case] expected: NumberOfCalls,
    ) {
        assert_eq!(
            evaluate_quota(max_quota, max_quota_percentage!(percentage)),
            expected,
            "evaluate_quota({}, {}%) should equal to {}",
            max_quota,
            percentage,
            expected,
        );
    }
}
