#![warn(clippy::pedantic)]

use std::{
    collections::HashMap,
    io::{self},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

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
pub trait Handler: Fn(Request) -> Response + Send + Sync + 'static {}

impl<F> Handler for F where F: Fn(Request) -> Response + Send + Sync + 'static {}

/// The main server data structure.
#[derive(Default)]
pub struct Server {
    /// Map from a route path (e.g., "/foo") to a handler function for that route.
    routes: HashMap<String, Box<dyn Handler>>,
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
        self.routes.insert(route.into(), Box::new(handler));
        self
    }

    /// Runs the server by listening for connections and returning responses.
    ///
    /// This function should never return.
    ///
    /// # Panics
    ///
    /// Panics if `127.0.0.1:3000` is not available.
    pub fn run(self) {
        let listener =
            TcpListener::bind("127.0.0.1:3000").expect("Failed to connect to 127.0.0.1:3000");
        let this = Arc::new(self);
        for stream in listener.incoming().flatten() {
            let this_ref = Arc::clone(&this);
            thread::spawn(move || {
                let _ = this_ref.handle(&stream);
            });
        }
    }

    fn handle(&self, stream: &TcpStream) -> io::Result<()> {
        protocol::handle(stream, |route, request| {
            self.routes.get(route).map(move |handler| handler(request))
        })
    }
}
