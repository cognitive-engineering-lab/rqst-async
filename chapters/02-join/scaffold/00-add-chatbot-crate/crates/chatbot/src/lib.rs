use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{cell::RefCell, time::Duration};

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
}

/// Seeds the thread-local RNG used by [`gen_random_number`].
pub fn seed_rng(seed: u64) {
    RNG.with(|rng| *rng.borrow_mut() = SmallRng::seed_from_u64(seed));
}

/// Generates a random `usize`.
///
/// Warning: may take a few seconds!
pub async fn gen_random_number() -> usize {
    tokio::time::sleep(Duration::from_secs(2)).await;
    RNG.with(|rng| rng.borrow_mut().gen())
}

/// Generates a list of possible responses given the current chat.
///
/// Warning: may take a few seconds!
pub async fn query_chat(_messages: &[String]) -> Vec<String> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    vec![
        "And how does that make you feel?".to_string(),
        "Interesting! Go on...".to_string(),
    ]
}
