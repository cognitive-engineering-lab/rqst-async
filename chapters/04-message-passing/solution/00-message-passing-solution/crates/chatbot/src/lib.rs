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

/// A chatbot that responds to inputs.
pub struct Chatbot {
    emojis: Vec<String>,
    emoji_counter: usize,
}

impl Chatbot {
    /// Creates a new chatbot that uses the provided emoji in its responses.
    pub fn new(emojis: Vec<String>) -> Self {
        Chatbot {
            emojis,
            emoji_counter: 0,
        }
    }

    /// Generates a list of possible responses given the current chat.
    ///
    /// Warning: may take a few seconds!
    pub async fn query_chat(&mut self, messages: &[String]) -> Vec<String> {
        std::thread::sleep(Duration::from_secs(2));
        let most_recent = messages.last().unwrap();
        let emoji = &self.emojis[self.emoji_counter];
        self.emoji_counter = (self.emoji_counter + 1) % self.emojis.len();
        vec![
            format!("\"{most_recent}\"? And how does that make you feel? {emoji}",),
            format!("\"{most_recent}\"! Interesting! Go on... {emoji}"),
        ]
    }
}
