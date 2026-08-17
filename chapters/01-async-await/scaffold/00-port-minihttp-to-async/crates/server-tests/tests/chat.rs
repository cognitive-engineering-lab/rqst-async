use serde_json::json;
use server_tests::Server;

#[test]
fn chat_gives_reply() {
    let server = Server::start();
    let (status, body) = server.post("/chat", r#"{"messages": ["Hello, world!"]}"#);
    assert_eq!(status, 200, "POST /chat responded {status}: {body:?}");

    let result =
        serde_json::from_str::<serde_json::Value>(&body).expect("/chat returned invalid JSON");
    assert_eq!(
        result,
        json!({
            "messages": ["Hello, world!", "And how does that make you feel?"]
        })
    );
}
