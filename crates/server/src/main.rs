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

async fn chat(Json(mut chat): Json<Chat>) -> Json<Chat> {
    let responses_fut = chatbot::query_chat(&chat.messages);
    let random_fut = chatbot::gen_random_number();
    let (mut responses, random) = join!(responses_fut, random_fut);
    let response = responses.remove(random % responses.len());
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
