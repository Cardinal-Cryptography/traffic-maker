/// A single token is 10^12 rappens.
pub const DECIMALS: u128 = 1_000_000_000_000;

/// As `toml` does not support deserializing `u128`, so we need to operate
/// on amounts scaled by `DECIMALS`.
pub const fn real_amount(scaled: u64) -> u128 {
    scaled as u128 * DECIMALS
}
