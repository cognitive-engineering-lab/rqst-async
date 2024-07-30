use std::{collections::HashMap, future::Future, io, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::ReusableBoxFuture;

/// Re-export for library clients.
pub use http;

/// Implementation details for HTTP.
mod protocol;

/// A request from a client, either a GET or a POST with a body.
#[derive(Debug, Clone)]
pub enum Request {
    Get,
    Post(String),
}

/// Content to give to a client, either HTML or JSON.
#[derive(Debug, Clone)]
pub enum Content {
    Html(String),
    Json(String),
}

/// Response to give to a client, either content or a status code for a failure (e.g. 404).
pub type Response = Result<Content, http::StatusCode>;

/// Trait alias for functions that can handle requests and return responses.
pub trait Handler: Fn(Request) -> Self::Future + Send + Sync + 'static {
    type Future: Future<Output = Response> + Send + Sync + 'static;
}

impl<F, H> Handler for H
where
    F: Future<Output = Response> + Send + Sync + 'static,
    H: Fn(Request) -> F + Send + Sync + 'static,
{
    type Future = F;
}

struct ErasedHandler(
    Box<dyn Fn(Request) -> ReusableBoxFuture<'static, Response> + Send + Sync + 'static>,
);

/// The main server data structure.
#[derive(Default)]
pub struct Server {
    /// Map from a route path (e.g., "/foo") to a handler function for that route.
    routes: HashMap<String, ErasedHandler>,
}

impl Server {
    /// Creates a server with no routes.
    pub fn new() -> Self {
        Server {
            routes: HashMap::new(),
        }
    }

    /// Adds a new route to the server.
    pub fn route<H: Handler>(mut self, route: impl Into<String>, handler: H) -> Self {
        let handler = Arc::new(handler);
        let erased = ErasedHandler(Box::new(move |req| {
            let handler_ref = Arc::clone(&handler);
            ReusableBoxFuture::new(async move { handler_ref(req).await })
        }));
        self.routes.insert(route.into(), erased);
        self
    }

    /// Runs the server by listening for connections and returning responses.
    ///
    /// This function should never return.
    pub async fn run(self) {
        let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
        let this = Arc::new(self);
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let this_ref = Arc::clone(&this);
                tokio::spawn(async move {
                    let _ = this_ref.handle(stream).await;
                });
            }
        }
    }

    async fn handle(&self, stream: TcpStream) -> io::Result<()> {
        protocol::handle(stream, &|route| self.routes.get(route)).await
    }
}
