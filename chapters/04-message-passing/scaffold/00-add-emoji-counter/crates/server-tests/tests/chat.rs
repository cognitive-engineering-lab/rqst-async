use chatbot::Chatbot;
use server_tests::Server;

#[test]
fn chat_gives_reply() {
    let server = Server::start();
    let (status, body) = server.post("/chat", r#"{"messages": ["Hello, world!"]}"#);
    assert_eq!(status, 200, "POST /chat responded {status}: {body:?}");

    let result =
        serde_json::from_str::<serde_json::Value>(&body).expect("/chat returned invalid JSON");
    let messages = result["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert!(messages[0].as_str().unwrap() == "Hello, world!");
}

#[tokio::test]
async fn chat_gives_known_reply() {
    let server = Server::start();

    let replies = Chatbot::new(vec!["".to_string()])
        .query_chat(&["Hello, world!".to_string()])
        .await;

    let (status, body) = server.post("/chat", r#"{"messages": ["Hello, world!"]}"#);
    assert_eq!(status, 200, "POST /chat responded {status}: {body:?}");

    let result =
        serde_json::from_str::<serde_json::Value>(&body).expect("/chat returned invalid JSON");
    let messages = result["messages"].as_array().unwrap();
    let reply = messages[1].as_str().unwrap().to_string();
    assert!(
        replies.iter().any(|ref_reply| reply.starts_with(ref_reply)),
        "chat does not give expected reply: {reply}"
    );
}
