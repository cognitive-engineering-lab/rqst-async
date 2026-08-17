use minihttp::{Content, Request, Response};

fn index(_req: Request) -> Response {
    let content = include_str!("../index.html").to_string();
    Ok(Content::Html(content))
}

fn main() {
    minihttp::Server::new().route("/", index).run()
}
