use miniserve::{Content, Request, Response};

fn index(_req: Request) -> Response {
    let content = include_str!("../index.html").to_string();
    Ok(Content::Html(content))
}

fn main() {
    miniserve::Server::new().route("/", index).run()
}
