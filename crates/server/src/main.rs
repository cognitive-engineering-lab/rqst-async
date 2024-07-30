use miniserve::{http::StatusCode, Content, Request, Response};
use serde::{Deserialize, Serialize};

fn index(_req: Request) -> Response {
    let content = include_str!("../index.html").to_string();
    Ok(Content::Html(content))
}

#[derive(Serialize, Deserialize)]
struct Messages {
    messages: Vec<String>,
}

fn chat(req: Request) -> Response {
    let Request::Post(body) = req else {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    };
    let Ok(mut messages) = serde_json::from_str::<Messages>(&body) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    messages
        .messages
        .push("And how does that make you feel?".into());
    Ok(Content::Json(serde_json::to_string(&messages).unwrap()))
}

fn main() {
    miniserve::Server::new()
        .route("/", index)
        .route("/chat", chat)
        .run()
}
