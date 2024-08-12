use std::sync::{Arc, LazyLock};

use miniserve::{http::StatusCode, Content, Request, Response};
use serde::{Deserialize, Serialize};
use tokio::{
    join,
    sync::{mpsc, oneshot},
};

async fn index(_req: Request) -> Response {
    let content = include_str!("../index.html").to_string();
    Ok(Content::Html(content))
}

#[derive(Serialize, Deserialize)]
struct Messages {
    messages: Vec<String>,
}

async fn query_chat(messages: &Arc<Vec<String>>) -> Vec<String> {
    type Payload = (Arc<Vec<String>>, oneshot::Sender<Vec<String>>);
    static SENDER: LazyLock<mpsc::Sender<Payload>> = LazyLock::new(|| {
        let (tx, mut rx) = mpsc::channel::<Payload>(1024);
        tokio::spawn(async move {
            let mut chatbot = chatbot::Chatbot::new(vec![":-)".into(), "^^".into()]);
            while let Some((messages, responder)) = rx.recv().await {
                let response = chatbot.query_chat(&messages).await;
                responder.send(response).unwrap();
            }
        });
        tx
    });

    let (tx, rx) = oneshot::channel();
    SENDER.send((Arc::clone(messages), tx)).await.unwrap();
    rx.await.unwrap()
}

async fn chat(req: Request) -> Response {
    let Request::Post(body) = req else {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    };
    let Ok(mut data) = serde_json::from_str::<Messages>(&body) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let messages = Arc::new(data.messages);
    let (i, mut responses) = join!(chatbot::gen_random_number(), query_chat(&messages));

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
