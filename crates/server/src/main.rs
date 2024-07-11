use anyhow::Result;
use axum::{
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

async fn root() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

#[derive(Serialize, Deserialize)]
struct Chat {
    messages: Vec<String>,
}

async fn chat(Json(mut chat): Json<Chat>) -> Json<Chat> {
    chat.messages
        .push("And how does that make you feel?".to_string());
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
