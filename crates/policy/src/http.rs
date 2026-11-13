//! The little HTTP/1.1 the endpoint speaks: one request per connection,
//! `POST /mcp` with a JSON body. Parsing is pure so it is tested here.

use std::collections::HashMap;

/// The longest request head the endpoint reads.
pub const MAX_HEAD_BYTES: usize = 16 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub struct Head {
    pub method: String,
    pub path: String,
    /// Header names in lower case; a repeated header keeps its last value.
    pub headers: HashMap<String, String>,
}

impl Head {
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }

    /// The declared body length; a missing header means no body.
    ///
    /// # Errors
    /// When the header is not a number.
    pub fn content_length(&self) -> Result<usize, &'static str> {
        self.header("content-length").map_or(Ok(0), |v| {
            v.trim().parse().map_err(|_| "bad Content-Length")
        })
    }
}

/// Where the head ends in `buf` (the index after the blank line), if it does.
#[must_use]
pub fn head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}

/// Parses a request head, the bytes up to and including the blank line.
///
/// # Errors
/// When the request line or a header line is malformed.
pub fn parse_head(head: &[u8]) -> Result<Head, &'static str> {
    let text = std::str::from_utf8(head).map_err(|_| "request head is not UTF-8")?;
    let mut lines = text.split("\r\n");
    let mut parts = lines.next().unwrap_or("").split(' ');
    let (Some(method), Some(path), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err("bad request line");
    };
    if !version.starts_with("HTTP/1.") || method.is_empty() || !path.starts_with('/') {
        return Err("bad request line");
    }
    let mut headers = HashMap::new();
    for line in lines.take_while(|l| !l.is_empty()) {
        let (name, value) = line.split_once(':').ok_or("bad header line")?;
        if name.is_empty() || name.contains(' ') {
            return Err("bad header line");
        }
        headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
    }
    Ok(Head {
        method: method.to_owned(),
        path: path.to_owned(),
        headers,
    })
}

/// A complete response with a JSON or text body; the connection closes after it.
#[must_use]
pub fn response(status: u16, content_type: &str, body: &str) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        _ => "Error",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQ: &[u8] = b"POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:8751\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";

    #[test]
    fn a_request_head_parses() {
        let end = head_end(REQ).unwrap();
        assert_eq!(&REQ[end..], b"{}");
        let head = parse_head(&REQ[..end]).unwrap();
        assert_eq!(head.method, "POST");
        assert_eq!(head.path, "/mcp");
        assert_eq!(head.header("host"), Some("127.0.0.1:8751"));
        assert_eq!(head.content_length(), Ok(2));
    }

    #[test]
    fn an_unfinished_head_has_no_end() {
        assert_eq!(head_end(b"POST /mcp HTTP/1.1\r\nHost: x\r\n"), None);
    }

    #[test]
    fn malformed_heads_are_refused() {
        for bad in [
            &b"GARBAGE\r\n\r\n"[..],
            b"POST mcp HTTP/1.1\r\n\r\n",
            b"POST /mcp SPDY/3\r\n\r\n",
            b"POST /mcp HTTP/1.1\r\nNoColon\r\n\r\n",
            b"POST /mcp HTTP/1.1 extra\r\n\r\n",
        ] {
            assert!(parse_head(bad).is_err(), "{}", String::from_utf8_lossy(bad));
        }
    }

    #[test]
    fn a_bad_content_length_is_an_error() {
        let head = parse_head(b"POST /mcp HTTP/1.1\r\nContent-Length: lots\r\n\r\n").unwrap();
        assert!(head.content_length().is_err());
    }

    #[test]
    fn a_response_states_its_length() {
        let r = String::from_utf8(response(200, "application/json", "{\"a\":1}")).unwrap();
        assert!(r.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(r.contains("Content-Length: 7\r\n"));
        assert!(r.ends_with("\r\n\r\n{\"a\":1}"));
    }
}
