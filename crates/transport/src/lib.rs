//! The loopback listener: one request per connection, admitted by
//! `policy::request`, answered by `policy::mcp`.

use std::io::{self, Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use policy::{http, mcp, request};

/// Where the endpoint listens and what it asks of a caller.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub token: Option<String>,
}

impl Config {
    /// Reads the caller-selected port and token variables from the environment.
    pub fn from_env(port_env: &str, token_env: &str, default_port: u16) -> Self {
        let port = std::env::var(port_env)
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(default_port);
        let token = std::env::var(token_env).ok().filter(|t| !t.is_empty());
        Self { port, token }
    }
}

/// Binds 127.0.0.1 and serves until the process ends. Only loopback is ever
/// bound: there is no setting for another address.
pub fn run(config: &Config, tools: &impl mcp::Tools, version: &str) {
    match TcpListener::bind(("127.0.0.1", config.port)) {
        Ok(listener) => {
            enshwick::info!(
                "MCP endpoint on http://127.0.0.1:{}/mcp{}",
                config.port,
                if config.token.is_some() {
                    " (token required)"
                } else {
                    ""
                }
            );
            serve(&listener, config, tools, version);
        }
        Err(e) => enshwick::error!("could not listen on 127.0.0.1:{}: {e}", config.port),
    }
}

/// Answers connections one at a time; a game read waits for the game thread
/// anyway, so parallel requests would only queue there.
pub fn serve(listener: &TcpListener, config: &Config, tools: &impl mcp::Tools, version: &str) {
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                if let Err(e) = answer(&mut s, config, tools, version) {
                    enshwick::warn!("request failed: {e}");
                }
            }
            Err(e) => enshwick::warn!("accept failed: {e}"),
        }
    }
}

fn answer(
    stream: &mut TcpStream,
    config: &Config,
    tools: &impl mcp::Tools,
    version: &str,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let (head, mut body) = match read_head(stream)? {
        Some(v) => v,
        None => return stream.write_all(&http::response(400, "text/plain", "bad request head")),
    };
    let length = match head.content_length() {
        Ok(n) => n,
        Err(e) => return stream.write_all(&http::response(400, "text/plain", e)),
    };
    let admission = request::Admission {
        origin: head.header("origin"),
        host: head.header("host"),
        authorization: head.header("authorization"),
        content_length: length,
    };
    if let Some(refusal) = request::reject(&admission, config.token.as_deref()) {
        return stream.write_all(&http::response(
            refusal.status(),
            "text/plain",
            refusal.message(),
        ));
    }
    if head.path != "/mcp" {
        return stream.write_all(&http::response(404, "text/plain", "the endpoint is /mcp"));
    }
    if head.method != "POST" {
        return stream.write_all(&http::response(
            405,
            "text/plain",
            "POST only (no SSE stream)",
        ));
    }
    if !mcp::header_acceptable(head.header("mcp-protocol-version")) {
        return stream.write_all(&http::response(
            400,
            "text/plain",
            "unsupported MCP-Protocol-Version",
        ));
    }
    while body.len() < length {
        let mut chunk = [0u8; 8192];
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "body shorter than Content-Length",
            ));
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(length);
    let Ok(text) = std::str::from_utf8(&body) else {
        return stream.write_all(&http::response(400, "text/plain", "body is not UTF-8"));
    };
    match mcp::handle(text, tools, version) {
        Some(reply) => {
            stream.write_all(&http::response(200, "application/json", &reply.to_string()))
        }
        None => stream.write_all(&http::response(202, "text/plain", "")),
    }
}

/// The parsed head and whatever body bytes came with it; `None` for a head
/// that is malformed or longer than `http::MAX_HEAD_BYTES`.
fn read_head(stream: &mut TcpStream) -> io::Result<Option<(http::Head, Vec<u8>)>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        if let Some(end) = http::head_end(&buf) {
            return Ok(http::parse_head(&buf[..end])
                .ok()
                .map(|h| (h, buf[end..].to_vec())));
        }
        if buf.len() > http::MAX_HEAD_BYTES {
            return Ok(None);
        }
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Ok(None);
        }
        buf.extend_from_slice(&chunk[..n]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use policy::mcp::{Tool, Tools};
    use serde_json::{Map, Value, json};

    struct One(Vec<Tool>);

    impl Tools for One {
        fn list(&self) -> &[Tool] {
            &self.0
        }
        fn call(&self, _: &str, _: &Map<String, Value>) -> Result<Value, String> {
            Ok(json!({"ok": true}))
        }
        fn present(
            &self,
            name: &str,
            value: Value,
        ) -> Result<policy::presentation::Presentation, String> {
            use policy::presentation::{Png, Presentation};
            if name != "picture" {
                return Ok(Presentation::Json(value));
            }
            let image = Png::rgb(
                2,
                3,
                &[
                    255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0, 0, 255, 255, 255, 0, 255,
                ],
            )?;
            Ok(Presentation::Png {
                metadata: json!({"label":"synthetic Δ","width":2,"height":3}),
                image,
            })
        }
    }

    /// Starts `serve` on a free loopback port and returns the port.
    fn start(token: Option<&str>) -> u16 {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let config = Config {
            port,
            token: token.map(Into::into),
        };
        std::thread::spawn(move || {
            let tools = One(vec![
                Tool {
                    name: "ok",
                    description: "",
                    schema: json!({"type": "object"}),
                },
                Tool {
                    name: "picture",
                    description: "synthetic image fixture",
                    schema: json!({"type":"object"}),
                },
            ]);
            serve(&listener, &config, &tools, "fixture-version");
        });
        port
    }

    fn send(port: u16, raw: &str) -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.write_all(raw.as_bytes()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    }

    fn post(body: &str, extra: &str) -> String {
        format!(
            "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\n{extra}Content-Length: {}\r\n\r\n{body}",
            body.len()
        )
    }

    #[test]
    fn initialize_reports_the_callers_version() {
        let port = start(None);
        let out = send(
            port,
            &post(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"fixture","version":"1"}}}"#,
                "",
            ),
        );
        assert!(out.starts_with("HTTP/1.1 200 OK"), "{out}");
        assert!(out.contains("fixture-version"), "{out}");
    }

    #[test]
    fn a_tool_call_round_trips() {
        let port = start(None);
        let out = send(
            port,
            &post(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"ok"}}"#,
                "",
            ),
        );
        assert!(out.starts_with("HTTP/1.1 200 OK"), "{out}");
        assert!(out.contains(r#"\"ok\": true"#), "{out}");
    }

    #[test]
    fn image_and_unicode_metadata_round_trip_with_exact_http_length() {
        let port = start(None);
        let out = send(
            port,
            &post(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"picture"}}"#,
                "",
            ),
        );
        let (head, body) = out.split_once("\r\n\r\n").unwrap();
        assert!(head.starts_with("HTTP/1.1 200 OK"));
        let length: usize = head
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(body.len(), length);
        let result: Value = serde_json::from_str(body).unwrap();
        let result = &result["result"];
        assert_eq!(result["isError"], false);
        let metadata: Value =
            serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(
            metadata,
            json!({"label":"synthetic Δ","width":2,"height":3})
        );
        assert_eq!(result["content"][1]["type"], "image");
        assert_eq!(result["content"][1]["mimeType"], "image/png");
        assert!(
            result["content"][1]["data"]
                .as_str()
                .unwrap()
                .starts_with("iVBORw0KGgo")
        );
    }

    #[test]
    fn a_notification_is_accepted_without_a_body() {
        let port = start(None);
        let out = send(
            port,
            &post(
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                "",
            ),
        );
        assert!(out.starts_with("HTTP/1.1 202"), "{out}");
    }

    #[test]
    fn browsers_and_wrong_tokens_are_refused() {
        let port = start(Some("s3cret"));
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        assert!(send(port, &post(body, "")).starts_with("HTTP/1.1 401"));
        assert!(
            send(port, &post(body, "Origin: http://example.com\r\n")).starts_with("HTTP/1.1 403")
        );
        assert!(
            send(port, &post(body, "Authorization: Bearer s3cret\r\n")).starts_with("HTTP/1.1 200")
        );
    }

    #[test]
    fn other_paths_and_methods_are_refused() {
        let port = start(None);
        assert!(
            send(port, "GET /mcp HTTP/1.1\r\nHost: localhost\r\n\r\n").starts_with("HTTP/1.1 405")
        );
        assert!(
            send(
                port,
                "POST /x HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n"
            )
            .starts_with("HTTP/1.1 404")
        );
        assert!(send(port, "nonsense\r\n\r\n").starts_with("HTTP/1.1 400"));
    }
}
