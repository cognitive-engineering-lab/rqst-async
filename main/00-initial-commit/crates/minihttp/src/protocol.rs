//! Implementation details for HTTP.
//!
//! You should not need to deal with this module.

use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::{Shutdown, TcpStream},
};

use http::StatusCode;

pub fn stringify_response(response: http::Response<Vec<u8>>) -> Vec<u8> {
    let (parts, body) = response.into_parts();

    let mut buf = Vec::with_capacity(body.len() + 256);
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

    buf
}

#[allow(clippy::result_large_err)]
fn parse_request(src: &[u8]) -> Result<Option<http::Request<Vec<u8>>>, http::Response<Vec<u8>>> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut parsed_req = httparse::Request::new(&mut headers);
    let Ok(status) = parsed_req.parse(src) else {
        return Err(make_response(
            StatusCode::BAD_REQUEST,
            "Failed to parse request",
        ));
    };
    let amt = match status {
        httparse::Status::Complete(amt) => amt,
        httparse::Status::Partial => return Ok(None),
    };

    let Ok(method) = http::Method::try_from(parsed_req.method.unwrap()) else {
        return Err(make_response(
            StatusCode::BAD_REQUEST,
            "Failed to parse request",
        ));
    };

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

fn make_response(status: http::StatusCode, explanation: &str) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .body(explanation.as_bytes().to_vec())
        .unwrap()
}

fn generate_response(
    req: http::Request<Vec<u8>>,
    callback: impl Fn(&str, crate::Request) -> Option<crate::Response>,
) -> http::Response<Vec<u8>> {
    let (parts, body) = req.into_parts();
    let request = match parts.method {
        http::Method::GET => crate::Request::Get,
        http::Method::POST => crate::Request::Post(String::from_utf8(body).unwrap()),
        _ => return make_response(StatusCode::METHOD_NOT_ALLOWED, "Not implemented"),
    };

    let Some(response_res) = callback(parts.uri.path(), request) else {
        return make_response(StatusCode::NOT_FOUND, "No valid route");
    };

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

pub fn handle(
    stream: &TcpStream,
    callback: impl Fn(&str, crate::Request) -> Option<crate::Response>,
) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream.try_clone()?);

    loop {
        let req = loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                stream.shutdown(Shutdown::Both)?;
                return Ok(());
            }

            match parse_request(buf) {
                Ok(None) => {}
                Ok(Some(req)) => {
                    let amt = buf.len();
                    reader.consume(amt);
                    break Ok(req);
                }
                Err(resp) => {
                    let amt = buf.len();
                    reader.consume(amt);
                    break Err(resp);
                }
            }
        };

        let resp = match req {
            Ok(req) => generate_response(req, &callback),
            Err(resp) => resp,
        };

        let buf = stringify_response(resp);
        writer.write_all(&buf)?;
        writer.flush()?;
    }
}
