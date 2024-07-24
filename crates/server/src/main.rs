use miniserve::{Content, Request, Response};

fn index(_req: Request) -> Response {
    Ok(Content::Html(include_str!("../index.html").into()))
}

fn main() {
    miniserve::Server::new().route("/", index).run()
}
