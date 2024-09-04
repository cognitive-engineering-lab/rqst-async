use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{cell::RefCell, io, path::PathBuf, time::Duration};

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
    let sleep_time = RNG.with(|rng| rng.borrow_mut().gen_range::<f32, _>(0. ..5.));
    tokio::time::sleep(Duration::from_secs_f32(sleep_time)).await;
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

    pub fn retrieval_documents(&self, _messages: &[String]) -> Vec<PathBuf> {
        vec![
            PathBuf::from("data/doc1.txt"),
            PathBuf::from("data/doc2.txt"),
        ]
    }

    /// Generates a list of possible responses given the current chat.
    ///
    /// Warning: may take a few seconds!
    pub async fn query_chat(&mut self, messages: &[String], docs: &[String]) -> Vec<String> {
        let most_recent = messages.last().unwrap();
        let sleep_time = RNG.with(|rng| rng.borrow_mut().gen_range::<f32, _>(0. ..5.));
        tokio::time::sleep(Duration::from_secs_f32(sleep_time)).await;
        let emoji = &self.emojis[self.emoji_counter];
        self.emoji_counter = (self.emoji_counter + 1) % self.emojis.len();
        vec![
            format!("\"{most_recent}\"? And how does that make you feel? {emoji}",),
            format!("\"{most_recent}\"! Interesting! Go on... {emoji}"),
            format!("Have you considered: {}", docs.first().unwrap()),
            format!("I might recommend: {}", docs.last().unwrap()),
        ]
    }
}

/// Holds chat messages and writes them to disk infrequently.
#[derive(Default)]
pub struct Logger {
    logs: Vec<String>,
}

impl Logger {
    /// Saves the message to the logger.
    pub fn append(&mut self, message: &str) {
        self.logs.push(message.to_string());
    }

    /// Potentially writes the logs to disk, if needed.
    pub async fn save(&self) -> io::Result<()> {
        if self.logs.len() % 3 == 0 {
            tokio::fs::write("log.txt", self.logs.join("\n")).await?;
        }
        Ok(())
    }
}
