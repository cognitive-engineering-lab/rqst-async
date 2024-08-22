//! Implementation details for HTTP.
//!
//! You should not need to deal with this module.

use futures::SinkExt;
use http::StatusCode;
use std::io;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder, Framed},
};

struct HttpCodec;

impl Encoder<http::Response<Vec<u8>>> for HttpCodec {
    type Error = io::Error;

    fn encode(
        &mut self,
        response: http::Response<Vec<u8>>,
        buf: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let (parts, body) = response.into_parts();

        buf.extend_from_slice(b"HTTP/1.1 ");
        buf.extend(parts.status.as_str().as_bytes());
        if let Some(reason) = parts.status.canonical_reason() {
            buf.extend_from_slice(b" ");
            buf.extend(reason.as_bytes());
        }

        buf.extend_from_slice(b"\r\n");

        for (name, value) in parts.headers {
            if let Some(name) = name {
                buf.extend(name.as_str().as_bytes());
                buf.extend_from_slice(b": ");
            }
            buf.extend(value.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        buf.extend_from_slice(b"\r\n");
        buf.extend(body);

        Ok(())
    }
}

impl Decoder for HttpCodec {
    type Item = http::Request<Vec<u8>>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut parsed_req = httparse::Request::new(&mut headers);
        let status = parsed_req.parse(src).map_err(|e| {
            let msg = format!("failed to parse http request: {e:?}");
            io::Error::new(io::ErrorKind::Other, msg)
        })?;
        let amt = match status {
            httparse::Status::Complete(amt) => amt,
            httparse::Status::Partial => return Ok(None),
        };

        let method = http::Method::try_from(parsed_req.method.unwrap()).map_err(|e| {
            let msg = format!("failed to parse http request: {e:?}");
            io::Error::new(io::ErrorKind::Other, msg)
        })?;

        let data = &src[amt..];

        let mut builder = http::Request::builder()
            .method(method)
            .version(http::Version::HTTP_11)
            .uri(parsed_req.path.unwrap());
        for header in parsed_req.headers {
            builder = builder.header(header.name, header.value);
        }

        Ok(Some(builder.body(data.to_vec()).unwrap()))
    }
}

fn make_response(status: http::StatusCode, explanation: &str) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .body(explanation.as_bytes().to_vec())
        .unwrap()
}

async fn generate_response<'a>(
    req: http::Request<Vec<u8>>,
    callback: impl Fn(&str) -> Option<&'a crate::ErasedHandler> + 'a,
) -> http::Response<Vec<u8>> {
    let (parts, body) = req.into_parts();
    let request = match parts.method {
        http::Method::GET => crate::Request::Get,
        http::Method::POST => crate::Request::Post(String::from_utf8(body).unwrap()),
        _ => return make_response(StatusCode::METHOD_NOT_ALLOWED, "Not implemented"),
    };

    let Some(handler) = callback(parts.uri.path()) else {
        return make_response(StatusCode::NOT_FOUND, "No valid route");
    };

    let response_res = handler(request).await;

    match response_res {
        Ok(content) => {
            let (body, ty) = match content {
                crate::Content::Html(body) => (body, "text/html"),
                crate::Content::Json(body) => (body, "application/json"),
            };
            http::Response::builder()
                .header("Content-Type", ty)
                .header("Content-Length", body.len())
                .body(body.into_bytes())
                .unwrap()
        }
        Err(status) => make_response(status, "Handler failed"),
    }
}

pub async fn handle<'a>(
    stream: TcpStream,
    callback: &'a (impl Fn(&str) -> Option<&'a crate::ErasedHandler> + 'a),
) -> io::Result<()> {
    let mut transport = Framed::new(stream, HttpCodec);
    if let Some(request) = transport.next().await {
        match request {
            Ok(request) => {
                let response = generate_response(request, callback).await;
                transport.send(response).await?;
            }
            Err(e) => {
                let response = make_response(StatusCode::BAD_REQUEST, &e.to_string());
                transport.send(response).await?;
            }
        }
    }

    Ok(())
}
