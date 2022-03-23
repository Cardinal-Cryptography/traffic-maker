/// A single token is 10^12 rappens.
pub const DECIMALS: u128 = 1_000_000_000_000;

/// As `toml` does not support deserializing `u128`, so we need to operate
/// on amounts scaled by `DECIMALS`.
pub const fn real_amount(scaled: u64) -> u128 {
    scaled as u128 * DECIMALS
}

pub fn secret_phrase_seed() -> String {
    option_env!("SECRET_PHRASE_SEED").unwrap_or("").to_string()
}

pub fn full_phrase(derivation: &str) -> String {
    format!("{}{}", secret_phrase_seed(), derivation)
}
