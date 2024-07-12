use std::sync::Arc;

use anyhow::Result;
use axum::{
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::join;

async fn root() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

#[derive(Serialize, Deserialize)]
struct Chat {
    messages: Vec<String>,
}

async fn chat(Json(chat): Json<Chat>) -> Json<Chat> {
    let chat_arc = Arc::new(chat);
    let chat_arc_ref = Arc::clone(&chat_arc);
    let responses_fut = tokio::spawn(async move { chatbot::query_chat(&chat_arc_ref.messages) });
    let random_fut = chatbot::gen_random_number();
    let (responses, random) = join!(responses_fut, random_fut);
    let mut responses = responses.unwrap();
    let response = responses.remove(random % responses.len());
    let mut chat = Arc::into_inner(chat_arc).unwrap();
    chat.messages.push(response);
    Json(chat)
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .route("/", get(root))
        .route("/chat", post(chat));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn chat_test() {
        chatbot::seed_rng(0);

        let mut chat_state = Chat {
            messages: Vec::new(),
        };

        for _ in 0..3 {
            chat_state = chat(Json(chat_state)).await.0;
        }

        assert_eq!(
            chat_state.messages,
            vec![
                "And how does that make you feel?",
                "Interesting! Go on...",
                "And how does that make you feel?"
            ]
        );
    }

    #[tokio::test]
    async fn chat_speed_test() {
        let chat_state = Chat {
            messages: Vec::new(),
        };

        let start = Instant::now();
        let _ = chat(Json(chat_state)).await;

        let elapsed = start.elapsed().as_secs_f32();
        assert!(
            elapsed < 3.,
            "Speed test took longer than 3s: {elapsed:.2}s"
        );
    }
}
