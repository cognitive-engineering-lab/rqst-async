#![warn(clippy::pedantic)]

use std::{collections::HashMap, future::Future, io, pin::Pin, sync::Arc};
use tokio::net::{TcpListener, TcpStream};

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

type ErasedHandler =
    Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + Sync>> + Send + Sync>;

/// The main server data structure.
#[derive(Default)]
pub struct Server {
    /// Map from a route path (e.g., "/foo") to a handler function for that route.
    routes: HashMap<String, ErasedHandler>,
}

impl Server {
    /// Creates a server with no routes.
    #[must_use]
    pub fn new() -> Self {
        Server {
            routes: HashMap::new(),
        }
    }

    /// Adds a new route to the server.
    #[must_use]
    pub fn route<H: Handler>(mut self, route: impl Into<String>, handler: H) -> Self {
        let handler = Arc::new(handler);
        let erased = Box::new(move |req| {
            let handler_ref = Arc::clone(&handler);
            Box::pin(handler_ref(req)) as Pin<Box<dyn Future<Output = Response> + Send + Sync>>
        });
        self.routes.insert(route.into(), erased);
        self
    }

    /// Runs the server by listening for connections and returning responses.
    ///
    /// This function should never return.
    ///
    /// # Panics
    ///
    /// Panics if `127.0.0.1:3000` is not available.
    pub async fn run(self) {
        let listener = TcpListener::bind("127.0.0.1:3000")
            .await
            .expect("Failed to connect to 127.0.0.1:3000");
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
