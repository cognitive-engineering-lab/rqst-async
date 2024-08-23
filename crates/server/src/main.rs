use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use miniserve::{http::StatusCode, Content, Request, Response};
use serde::{Deserialize, Serialize};
use stateful::StatefulThread;
use tokio::{fs, join, task::JoinSet};

mod stateful;

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

struct LogFunction {
    logger: chatbot::Logger,
}

impl stateful::StatefulFunction for LogFunction {
    type Input = Arc<Vec<String>>;
    type Output = ();

    async fn call(&mut self, messages: Self::Input) -> Self::Output {
        self.logger.append(messages.last().unwrap());
        self.logger.save().await.unwrap();
    }
}

static LOG_THREAD: LazyLock<StatefulThread<LogFunction>> = LazyLock::new(|| {
    StatefulThread::new(LogFunction {
        logger: chatbot::Logger::default(),
    })
});

struct ChatbotFunction {
    chatbot: chatbot::Chatbot,
}

impl stateful::StatefulFunction for ChatbotFunction {
    type Input = Arc<Vec<String>>;
    type Output = Vec<String>;

    async fn call(&mut self, messages: Self::Input) -> Self::Output {
        let doc_paths = self.chatbot.retrieval_documents(&messages);
        let docs = load_docs(doc_paths).await;
        self.chatbot.query_chat(&messages, &docs).await
    }
}

static CHATBOT_THREAD: LazyLock<StatefulThread<ChatbotFunction>> = LazyLock::new(|| {
    StatefulThread::new(ChatbotFunction {
        chatbot: chatbot::Chatbot::new(vec![":-)".into(), "^^".into()]),
    })
});

async fn cancel(_req: Request) -> Response {
    CHATBOT_THREAD.cancel().await;
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
    let (i, responses_opt, _) = join!(
        chatbot::gen_random_number(),
        CHATBOT_THREAD.call(Arc::clone(&messages)),
        LOG_THREAD.call(Arc::clone(&messages))
    );

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
