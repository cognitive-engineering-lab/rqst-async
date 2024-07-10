use std::time::Duration;
use tokio::time::sleep;

/// Generates a random `usize`.
///
/// Warning: may take a few seconds!
pub async fn gen_random_number() -> usize {
    sleep(Duration::from_secs(2)).await;
    4
}

/// Generates a list of possible responses given the current chat.
///
/// Warning: may take a few seconds!
pub async fn query_chat(_messages: &[String]) -> Vec<String> {
    sleep(Duration::from_secs(2)).await;
    vec![
        "And how does that make you feel?".to_string(),
        "Interesting! Go on...".to_string(),
    ]
}
