//! MCP over JSON-RPC 2.0: `initialize`, `ping`, `tools/list`, `tools/call`.
//! The game half supplies the tools through [`Tools`]; everything else is
//! decided here.

use crate::presentation::Presentation;
use serde_json::{Map, Value, json};

/// Protocol revisions answered, newest first. Only tools are implemented,
/// which all of them share.
pub const PROTOCOLS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

#[derive(Debug)]
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: Value,
}

/// What the game half provides.
pub trait Tools {
    fn list(&self) -> &[Tool];
    /// Runs a tool whose arguments already passed [`crate::args::check`].
    /// `Ok` is the tool's result as JSON, `Err` a message for the caller.
    ///
    /// # Errors
    /// When the tool fails or the game cannot answer.
    fn call(&self, name: &str, args: &Map<String, Value>) -> Result<Value, String>;
    /// Present an already successful result using copied, bounded data only.
    /// This hook grants no World access or native execution authority. Existing
    /// tools keep their JSON text format unless explicitly overridden.
    /// # Errors
    /// A read-only image producer may refuse rendering; no partial content is returned.
    fn present(&self, _name: &str, value: Value) -> Result<Presentation, String> {
        Ok(Presentation::Json(value))
    }
}

/// The negotiated revision: the client's when supported, else the newest.
#[must_use]
pub fn negotiate(requested: &str) -> &'static str {
    PROTOCOLS
        .iter()
        .find(|v| **v == requested)
        .copied()
        .unwrap_or(PROTOCOLS[0])
}

/// An absent `MCP-Protocol-Version` header is fine (old clients); an
/// unsupported one is answered with 400.
#[must_use]
pub fn header_acceptable(header: Option<&str>) -> bool {
    header.is_none_or(|h| h.trim().is_empty() || PROTOCOLS.contains(&h.trim()))
}

/// Answers one JSON-RPC message. `None` for a notification, which gets no
/// answer and is never run.
#[must_use]
pub fn handle(body: &str, tools: &impl Tools, server_version: &str) -> Option<Value> {
    let Ok(msg) = serde_json::from_str::<Value>(body) else {
        return Some(error(&Value::Null, -32700, "parse error"));
    };
    let Some(req) = msg.as_object() else {
        return Some(error(
            &Value::Null,
            -32600,
            "expected one JSON-RPC request object",
        ));
    };
    let id = req.get("id")?.clone();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let empty = Map::new();
    let params = req
        .get("params")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let result = match method {
        "initialize" => json!({
            "protocolVersion": negotiate(params.get("protocolVersion").and_then(Value::as_str).unwrap_or("")),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "enshrouded-mcp", "version": server_version},
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": tools.list().iter().map(|t| json!({
            "name": t.name, "description": t.description, "inputSchema": t.schema,
        })).collect::<Vec<_>>()}),
        "tools/call" => call(params, tools),
        _ => return Some(error(&id, -32601, &format!("method not found: {method}"))),
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

fn call(params: &Map<String, Value>, tools: &impl Tools) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let empty = Map::new();
    let args = match params.get("arguments") {
        None => &empty,
        Some(Value::Object(args)) => args,
        Some(_) => return text("arguments must be an object", true),
    };
    let Some(tool) = tools.list().iter().find(|t| t.name == name) else {
        return text(&format!("unknown tool: {name}"), true);
    };
    match crate::args::check(&tool.schema, args)
        .and_then(|()| tools.call(name, args))
        .and_then(|value| tools.present(name, value))
    {
        Ok(value) => value.result(),
        Err(e) => text(&e, true),
    }
}

fn text(text: &str, is_error: bool) -> Value {
    json!({"content": [{"type": "text", "text": text}], "isError": is_error})
}

fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Echo {
        tools: Vec<Tool>,
        calls: Cell<u32>,
    }

    impl Echo {
        fn new() -> Self {
            let schema = json!({"type": "object", "properties": {"x": {"type": "number"}}, "required": ["x"]});
            Self {
                tools: vec![Tool {
                    name: "echo",
                    description: "returns x",
                    schema,
                }],
                calls: Cell::new(0),
            }
        }
    }

    impl Tools for Echo {
        fn list(&self) -> &[Tool] {
            &self.tools
        }
        fn call(&self, _: &str, args: &Map<String, Value>) -> Result<Value, String> {
            self.calls.set(self.calls.get() + 1);
            if args["x"] == json!(13) {
                Err("unlucky".into())
            } else {
                Ok(json!({"x": args["x"]}))
            }
        }
    }

    fn ask(echo: &Echo, body: Value) -> Value {
        handle(&body.to_string(), echo, "0.1.0").expect("an answer")
    }

    #[test]
    fn initialize_negotiates_the_version() {
        let echo = Echo::new();
        let r = ask(
            &echo,
            json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}}),
        );
        assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(r["result"]["serverInfo"]["name"], "enshrouded-mcp");
        let r = ask(
            &echo,
            json!({"jsonrpc": "2.0", "id": "a", "method": "initialize", "params": {"protocolVersion": "1999-01-01"}}),
        );
        assert_eq!(r["result"]["protocolVersion"], PROTOCOLS[0]);
        assert_eq!(r["id"], "a", "the id keeps its type");
    }

    #[test]
    fn a_notification_gets_no_answer_and_runs_nothing() {
        let echo = Echo::new();
        let body = json!({"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "echo", "arguments": {"x": 1}}});
        assert_eq!(handle(&body.to_string(), &echo, "0"), None);
        assert_eq!(echo.calls.get(), 0);
    }

    #[test]
    fn tools_are_listed_with_their_schema() {
        let r = ask(
            &Echo::new(),
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        );
        assert_eq!(r["result"]["tools"][0]["name"], "echo");
        assert_eq!(r["result"]["tools"][0]["inputSchema"]["required"][0], "x");
    }

    #[test]
    fn a_call_runs_the_tool() {
        let r = ask(
            &Echo::new(),
            json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "echo", "arguments": {"x": 4}}}),
        );
        assert_eq!(r["result"]["isError"], false);
        assert_eq!(
            r["result"],
            json!({"content":[{"type":"text","text":"{\n  \"x\": 4\n}"}],"isError":false})
        );
    }

    #[test]
    fn bad_arguments_never_reach_the_tool() {
        let echo = Echo::new();
        let r = ask(
            &echo,
            json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "echo", "arguments": {"x": 1, "y": 2}}}),
        );
        assert_eq!(r["result"]["isError"], true);
        let r2 = ask(
            &echo,
            json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "echo", "arguments": {}}}),
        );
        assert_eq!(r2["result"]["isError"], true);
        assert_eq!(echo.calls.get(), 0);
    }

    #[test]
    fn malformed_arguments_never_become_an_empty_call() {
        let mut echo = Echo::new();
        echo.tools[0].schema = json!({"type": "object"});
        for arguments in [Value::Null, json!([]), json!("{}"), json!(false), json!(3)] {
            let r = ask(
                &echo,
                json!({"jsonrpc": "2.0", "id": 1,
                "method": "tools/call", "params": {"name": "echo", "arguments": arguments}}),
            );
            assert_eq!(r["result"]["isError"], true);
            assert_eq!(
                r["result"]["content"][0]["text"],
                "arguments must be an object"
            );
        }
        assert_eq!(echo.calls.get(), 0);
    }

    #[test]
    fn tool_failures_and_unknown_tools_are_tool_errors() {
        let echo = Echo::new();
        let r = ask(
            &echo,
            json!({"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "echo", "arguments": {"x": 13}}}),
        );
        assert_eq!(r["result"]["content"][0]["text"], "unlucky");
        let r = ask(
            &echo,
            json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "nope"}}),
        );
        assert_eq!(r["result"]["isError"], true);
    }

    #[test]
    fn protocol_errors_use_json_rpc_codes() {
        let echo = Echo::new();
        assert_eq!(
            handle("{nope", &echo, "0").unwrap()["error"]["code"],
            -32700
        );
        assert_eq!(
            handle("[1,2]", &echo, "0").unwrap()["error"]["code"],
            -32600
        );
        assert_eq!(
            ask(
                &echo,
                json!({"jsonrpc": "2.0", "id": 8, "method": "resources/list"})
            )["error"]["code"],
            -32601
        );
    }

    #[test]
    fn the_version_header_is_checked() {
        assert!(header_acceptable(None));
        assert!(header_acceptable(Some("2025-06-18")));
        assert!(!header_acceptable(Some("2030-01-01")));
    }

    struct Picture {
        echo: Echo,
        presentations: Cell<u32>,
    }
    impl Picture {
        fn new() -> Self {
            Self {
                echo: Echo::new(),
                presentations: Cell::new(0),
            }
        }
        fn ask(&self, params: Value) -> Value {
            handle(
                &json!({"jsonrpc":"2.0","id":31,"method":"tools/call","params":params}).to_string(),
                self,
                "fixture",
            )
            .unwrap()["result"]
                .clone()
        }
    }
    impl Tools for Picture {
        fn list(&self) -> &[Tool] {
            self.echo.list()
        }
        fn call(&self, name: &str, args: &Map<String, Value>) -> Result<Value, String> {
            self.echo.call(name, args)
        }
        fn present(&self, _: &str, value: Value) -> Result<Presentation, String> {
            self.presentations.set(self.presentations.get() + 1);
            if value["x"] == 9 {
                return Err("fixture presentation refused".into());
            }
            let width = if value["x"] == 8 { 0 } else { 1 };
            let image = crate::presentation::Png::rgb(width, 1, &[255, 0, 0])?;
            Ok(Presentation::Png {
                metadata: value,
                image,
            })
        }
    }

    #[test]
    fn png_content_is_explicit_and_follows_exactly_one_successful_call() {
        for protocol in PROTOCOLS {
            let picture = Picture::new();
            let init = handle(
                &json!({"id":1,"method":"initialize","params":{"protocolVersion":protocol}})
                    .to_string(),
                &picture,
                "fixture",
            )
            .unwrap();
            assert_eq!(init["result"]["protocolVersion"], protocol);
            let result = picture.ask(json!({"name":"echo","arguments":{"x":1}}));
            assert_eq!(picture.echo.calls.get(), 1);
            assert_eq!(picture.presentations.get(), 1);
            assert_eq!(result["isError"], false);
            assert_eq!(result["content"].as_array().unwrap().len(), 2);
            assert_eq!(result["content"][0]["type"], "text");
            assert_eq!(result["content"][1]["mimeType"], "image/png");
            assert!(
                result["content"][1]["data"]
                    .as_str()
                    .unwrap()
                    .starts_with("iVBORw0KGgo")
            );
        }
    }

    #[test]
    fn admission_errors_and_notifications_never_reach_presentation() {
        let picture = Picture::new();
        for params in [
            json!({"name":"missing"}),
            json!({"name":"echo","arguments":{"x":1,"unknown":0}}),
            json!({"name":"echo","arguments":[]}),
        ] {
            assert_eq!(picture.ask(params)["isError"], true);
        }
        assert!(
            handle(
                r#"{"method":"tools/call","params":{"name":"echo","arguments":{"x":1}}}"#,
                &picture,
                "fixture"
            )
            .is_none()
        );
        assert_eq!(picture.echo.calls.get(), 0);
        assert_eq!(picture.presentations.get(), 0);
    }

    #[test]
    fn tool_errors_never_render_or_promote_partial_results() {
        let picture = Picture::new();
        let result = picture.ask(json!({"name":"echo","arguments":{"x":13}}));
        assert_eq!(picture.echo.calls.get(), 1);
        assert_eq!(picture.presentations.get(), 0);
        assert_eq!(result["isError"], true);
        assert_eq!(result["content"].as_array().unwrap().len(), 1);
        assert_eq!(result["content"][0]["text"], "unlucky");
    }

    #[test]
    fn rendering_or_encoding_refusal_has_no_partial_image_or_implicit_retry() {
        for (x, message) in [(8, "PNG dimensions"), (9, "fixture presentation refused")] {
            let picture = Picture::new();
            let result = picture.ask(json!({"name":"echo","arguments":{"x":x}}));
            assert_eq!(picture.echo.calls.get(), 1);
            assert_eq!(picture.presentations.get(), 1);
            assert_eq!(result["isError"], true);
            assert_eq!(result["content"].as_array().unwrap().len(), 1);
            assert!(
                result["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .contains(message)
            );
        }
    }
}
