use std::sync::Arc;

use miniserve::{http::StatusCode, Content, Request, Response};
use serde::{Deserialize, Serialize};
use tokio::join;

async fn index(_req: Request) -> Response {
    let content = include_str!("../index.html").to_string();
    Ok(Content::Html(content))
}

#[derive(Serialize, Deserialize)]
struct Messages {
    messages: Vec<String>,
}

async fn chat(req: Request) -> Response {
    let Request::Post(body) = req else {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    };
    let Ok(mut data) = serde_json::from_str::<Messages>(&body) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let messages = Arc::new(data.messages);
    let messages_ref = Arc::clone(&messages);
    let (i, responses) = join!(
        chatbot::gen_random_number(),
        tokio::spawn(async move { chatbot::query_chat(&messages_ref).await })
    );
    let mut responses = responses.unwrap();

    let response = responses.remove(i % responses.len());
    data.messages = Arc::into_inner(messages).unwrap();
    data.messages.push(response);

    Ok(Content::Json(serde_json::to_string(&data).unwrap()))
}

#[tokio::main]
async fn main() {
    miniserve::Server::new()
        .route("/", index)
        .route("/chat", chat)
        .run()
        .await
}
