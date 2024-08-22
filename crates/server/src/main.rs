use std::{
    path::PathBuf,
    pin::pin,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

use miniserve::{http::StatusCode, Content, Request, Response};
use serde::{Deserialize, Serialize};
use tokio::{
    fs, join,
    sync::{mpsc, oneshot},
    task::JoinSet,
};

async fn index(_req: Request) -> Response {
    let content = include_str!("../index.html").to_string();
    Ok(Content::Html(content))
}

#[derive(Serialize, Deserialize)]
struct MessagesRequest {
    messages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum MessagesResponse {
    Success { messages: Vec<String> },
    Cancelled,
}

async fn load_docs(paths: Vec<PathBuf>) -> Vec<String> {
    let mut doc_futs = paths
        .into_iter()
        .map(fs::read_to_string)
        .collect::<JoinSet<_>>();
    let mut docs = Vec::new();
    while let Some(result) = doc_futs.join_next().await {
        docs.push(result.unwrap().unwrap());
    }
    docs
}

type Payload = (Arc<Vec<String>>, oneshot::Sender<Option<Vec<String>>>);

fn chatbot_thread() -> (mpsc::Sender<Payload>, mpsc::Sender<()>) {
    let (req_tx, mut req_rx) = mpsc::channel::<Payload>(1024);
    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
    tokio::spawn(async move {
        let mut chatbot = chatbot::Chatbot::new(vec![":-)".into(), "^^".into()]);
        while let Some((messages, responder)) = req_rx.recv().await {
            let doc_paths = chatbot.retrieval_documents(&messages);
            let docs = load_docs(doc_paths).await;
            let mut chat_fut = pin!(chatbot.query_chat(&messages, &docs));
            let mut cancel_fut = pin!(cancel_rx.recv());
            let start = Instant::now();
            loop {
                let log_fut = tokio::time::sleep(Duration::from_secs(1));
                tokio::select! {
                    response = &mut chat_fut => {
                        responder.send(Some(response)).unwrap();
                        break;
                    }
                    _ = &mut cancel_fut => {
                        responder.send(None).unwrap();
                        break;
                    }
                    _ = log_fut => {
                        println!("Waiting for {} seconds", start.elapsed().as_secs());
                    }
                }
            }
        }
    });
    (req_tx, cancel_tx)
}

static CHATBOT_THREAD: LazyLock<(mpsc::Sender<Payload>, mpsc::Sender<()>)> =
    LazyLock::new(chatbot_thread);

async fn query_chat(messages: &Arc<Vec<String>>) -> Option<Vec<String>> {
    let (tx, rx) = oneshot::channel();
    CHATBOT_THREAD
        .0
        .send((Arc::clone(messages), tx))
        .await
        .unwrap();
    rx.await.unwrap()
}

async fn cancel(_req: Request) -> Response {
    CHATBOT_THREAD.1.send(()).await.unwrap();
    Ok(Content::Html("success".into()))
}

async fn chat(req: Request) -> Response {
    let Request::Post(body) = req else {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    };
    let Ok(mut data) = serde_json::from_str::<MessagesRequest>(&body) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let messages = Arc::new(data.messages);
    let (i, responses_opt) = join!(chatbot::gen_random_number(), query_chat(&messages));

    let response = match responses_opt {
        Some(mut responses) => {
            let response = responses.remove(i % responses.len());
            data.messages = Arc::into_inner(messages).unwrap();
            data.messages.push(response);

            MessagesResponse::Success {
                messages: data.messages,
            }
        }
        None => MessagesResponse::Cancelled,
    };

    Ok(Content::Json(serde_json::to_string(&response).unwrap()))
}

#[tokio::main]
async fn main() {
    miniserve::Server::new()
        .route("/", index)
        .route("/chat", chat)
        .route("/cancel", cancel)
        .run()
        .await
}
