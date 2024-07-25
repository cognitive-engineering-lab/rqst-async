//! Implementation details for HTTP.
//!
//! You should not need to deal with this module.

use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
};

use http::StatusCode;

pub fn stringify_response(response: http::Response<Vec<u8>>) -> Vec<u8> {
    let (parts, body) = response.into_parts();

    let mut response_buf = Vec::new();
    write!(&mut response_buf, "HTTP/1.1 {}", parts.status.as_str()).unwrap();
    if let Some(reason) = parts.status.canonical_reason() {
        write!(&mut response_buf, " {reason}").unwrap();
    }
    write!(&mut response_buf, "\r\n").unwrap();

    for (name, value) in parts.headers {
        if let Some(name) = name {
            write!(&mut response_buf, "{}: ", name.as_str()).unwrap();
        }
        response_buf.extend(value.as_bytes());
        write!(&mut response_buf, "\r\n").unwrap();
    }

    write!(&mut response_buf, "\r\n").unwrap();
    response_buf.extend(body);

    response_buf
}

fn read_request<'buf>(
    stream: &mut TcpStream,
    req_buf: &'buf mut Vec<u8>,
    req: &mut httparse::Request<'_, 'buf>,
    body_buf: &mut Vec<u8>,
) -> Result<(), http::Response<Vec<u8>>> {
    let reader = &mut BufReader::new(stream);
    for line_res in reader.lines() {
        match line_res {
            Ok(line) => {
                req_buf.extend(line.as_bytes());
                req_buf.extend(b"\r\n");
                if line.is_empty() {
                    break;
                }
            }
            Err(_) => {
                return Err(make_response(
                    StatusCode::BAD_REQUEST,
                    "Failed to read stream",
                ))
            }
        }
    }

    match req.parse(req_buf) {
        Err(_) => {
            return Err(make_response(
                StatusCode::BAD_REQUEST,
                "Failed to parse request",
            ))
        }
        Ok(status) if status.is_partial() => {
            return Err(make_response(StatusCode::BAD_REQUEST, "Partial request"));
        }
        _ => {}
    }

    let content_length = req
        .headers
        .iter()
        .find(|header| header.name == "Content-Length");
    if let Some(header) = content_length {
        let Ok(length) = String::from_utf8_lossy(header.value).parse::<usize>() else {
            return Err(make_response(
                StatusCode::BAD_REQUEST,
                "Invalid content length",
            ));
        };

        body_buf.clear();
        body_buf.extend((0..length).map(|_| 0));
        if reader.read_exact(body_buf).is_err() {
            return Err(make_response(StatusCode::BAD_REQUEST, "Invalid body"));
        }
    };

    Ok(())
}

fn make_response(status: http::StatusCode, explanation: &str) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .body(explanation.as_bytes().to_vec())
        .unwrap()
}

fn generate_response(
    stream: &mut TcpStream,
    callback: impl Fn(&str, crate::Request) -> Option<crate::Response>,
) -> http::Response<Vec<u8>> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    let mut req_buf = Vec::new();
    let mut body_buf = Vec::new();

    if let Err(resp) = read_request(stream, &mut req_buf, &mut req, &mut body_buf) {
        return resp;
    }

    let Some(path) = req.path else {
        return make_response(StatusCode::BAD_REQUEST, "Missing path");
    };

    let request = match req.method {
        Some("GET") => crate::Request::Get,
        Some("POST") => crate::Request::Post(String::from_utf8(body_buf).unwrap()),
        _ => return make_response(StatusCode::METHOD_NOT_ALLOWED, "Not implemented"),
    };

    let Some(response_res) = callback(path, request) else {
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
    mut stream: TcpStream,
    callback: impl Fn(&str, crate::Request) -> Option<crate::Response>,
) -> io::Result<()> {
    let response = generate_response(&mut stream, callback);
    let buf = stringify_response(response);
    stream.write_all(&buf)
}
