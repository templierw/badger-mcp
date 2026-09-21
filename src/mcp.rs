use crate::sp::Task;

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Request {
    pub jsonrpc: String,
    pub id: Option<Id>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Id {
    Number(u64),
    String(String),
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl Response {
    pub fn success(id: Id, result: serde_json::Value) -> Self {
        Response {
            jsonrpc: "2.0".to_owned(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Id, code: i64, message: String) -> Self {
        Response {
            jsonrpc: "2.0".to_owned(),
            id,
            result: None,
            error: Some(RpcError { code, message }),
        }
    }
}

#[derive(serde::Serialize, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        ServerInfo {
            name: "badger-mcp".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

pub fn handle(req: Request) -> Option<Response> {
    let id = req.id?;
    Some(match req.method.as_str() {
        "initialize" => {
            let requested_version = req
                .params
                .as_ref()
                .and_then(|params| params["protocolVersion"].as_str());
            match requested_version {
                Some(v) => Response::success(
                    id,
                    serde_json::json!(
                        {
                            "protocolVersion": v,
                            "capabilities": {"tools": {}},
                            "serverInfo": ServerInfo::default(),
                        }
                    ),
                ),
                None => Response::error(id, -32602, "Invalid params".to_owned()),
            }
        }
        "tools/list" => Response::success(
            id,
            serde_json::json!(
                {
                    "tools": [
                        {
                            "name": "list_tasks",
                            "description": "Return task titles, time estimates (in min) and project ids. Estimate can be missing.",
                            "inputSchema": {
                                "type": "object"
                            }
                        }
                    ]
                }
            ),
        ),
        "tools/call" => {
            let name = req
                .params
                .as_ref()
                .and_then(|params| params["name"].as_str());
            match name {
                Some(n) => Response::error(id, -32602, format!("Unknown tool: {:}", n)),
                None => Response::error(id, -32602, "Missing tool name".to_owned()),
            }
        }
        _ => Response::error(id, -32601, "Method not found".to_owned()),
    })
}

pub fn list_tasks_result(tasks: &[Task]) -> serde_json::Value {
    let text = tasks
        .iter()
        .map(|t| match t.time_estimate {
            Some(value) => format!("{} - {} - {}", t.title, value / 60_000, t.project_id),
            None => format!("{} - {}", t.title, t.project_id),
        })
        .collect::<Vec<String>>()
        .join("\n");
    serde_json::json!({ "content": [ {"type": "text", "text": text} ], "isError": false })
}

#[cfg(test)]
mod tests {
    use crate::mcp::{Id, Request, Response, RpcError, handle, list_tasks_result};
    use crate::sp::Task;

    #[test]
    fn a_request_exposes_its_method() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;

        let request: Request = serde_json::from_str(json).expect("should deserialize");

        assert_eq!(request.method, "tools/list");
    }

    #[test]
    fn a_notification_has_no_id() {
        let json = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

        let request: Request = serde_json::from_str(json).expect("should deserialize");

        assert_eq!(request.method, "notifications/initialized");
        assert!(request.id.is_none());
    }

    #[test]
    fn an_id_can_be_a_string() {
        let json = r#"{"jsonrpc":"2.0","id":"init-1","method":"initialize"}"#;

        let request: Request = serde_json::from_str(json).expect("should deserialize");

        assert_eq!(request.method, "initialize");
        assert_eq!(request.id, Some(Id::String("init-1".to_owned())));
    }

    #[test]
    fn a_success_response_echoes_the_id() {
        let response = Response {
            jsonrpc: "2.0".to_owned(),
            id: Id::Number(7),
            result: Some(serde_json::json!({"tools": []})),
            error: None,
        };

        let value = serde_json::to_value(&response).expect("should serialize");

        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["id"], 7);
        assert_eq!(value["result"]["tools"], serde_json::json!([]));
        assert!(value.get("error").is_none());
    }

    #[test]
    fn an_error_response_carries_a_code_and_no_result() {
        let response = Response {
            jsonrpc: "2.0".to_owned(),
            id: Id::Number(7),
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: "Method not found".to_owned(),
            }),
        };

        let value = serde_json::to_value(&response).expect("should serialize");

        assert_eq!(value["error"]["code"], -32601);
        assert_eq!(value["error"]["message"], "Method not found");
        assert!(value.get("result").is_none());
    }

    #[test]
    fn an_unknown_method_is_a_method_not_found_error() {
        let request: Request =
            serde_json::from_str(r#"{"jsonrpc":"2.0","id":7,"method":"badger/dance"}"#)
                .expect("should deserialize");

        let response = handle(request).expect("a request must get a response");

        assert_eq!(response.id, Id::Number(7));

        let value = serde_json::to_value(&response).expect("should serialize");
        assert_eq!(value["error"]["code"], -32601);
        assert_eq!(value["error"]["message"], "Method not found");
        assert!(value.get("result").is_none());
    }

    #[test]
    fn initialize_answers_the_handshake() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "claude-code", "version": "1.0"}
            }
        }"#;
        let request: Request = serde_json::from_str(json).expect("should deserialize");

        let response = handle(request).expect("a request must get a response");
        let value = serde_json::to_value(&response).expect("should serialize");

        assert!(value.get("error").is_none());
        assert_eq!(value["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(value["result"]["serverInfo"]["name"], "badger-mcp");
        assert!(value["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn tools_list_offers_list_tasks() {
        let request: Request =
            serde_json::from_str(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#)
                .expect("should deserialize");

        let response = handle(request).expect("a request must get a response");
        let value = serde_json::to_value(&response).expect("should serialize");

        let tools = value["result"]["tools"]
            .as_array()
            .expect("result.tools should be an array");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "list_tasks");
        assert!(
            tools[0]["description"]
                .as_str()
                .is_some_and(|d| !d.is_empty())
        );
        assert_eq!(tools[0]["inputSchema"]["type"], "object");
    }

    #[test]
    fn a_notification_gets_no_response() {
        let request: Request =
            serde_json::from_str(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
                .expect("should deserialize");

        assert!(handle(request).is_none());
    }

    #[test]
    fn calling_an_unknown_tool_is_an_invalid_params_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "no_such_tool", "arguments": {}}
        }"#;
        let request: Request = serde_json::from_str(json).expect("should deserialize");

        let response = handle(request).expect("a request must get a response");
        let value = serde_json::to_value(&response).expect("should serialize");

        assert_eq!(value["error"]["code"], -32602);
        assert!(
            value["error"]["message"]
                .as_str()
                .is_some_and(|m| m.contains("no_such_tool"))
        );
        assert!(value.get("result").is_none());
    }

    #[test]
    fn list_tasks_result_reports_estimates_in_minutes() {
        // No digits anywhere in this fixture, so any '0' in the rendered text
        // means a raw millisecond value leaked or a missing estimate became zero.
        let tasks = vec![
            Task {
                title: "check dynatrace access".to_owned(),
                time_estimate: Some(900_000),
                project_id: "proj_abc".to_owned(),
            },
            Task {
                title: "triage inbox".to_owned(),
                time_estimate: None,
                project_id: "proj_abc".to_owned(),
            },
        ];

        let result = list_tasks_result(&tasks);

        assert_eq!(result["isError"], false);

        let content = result["content"]
            .as_array()
            .expect("result.content should be an array");
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");

        let text = content[0]["text"]
            .as_str()
            .expect("the content item should carry text");
        assert!(text.contains("check dynatrace access"));
        assert!(text.contains("triage inbox"));
        assert!(text.contains("15"));
        assert!(!text.contains('0'));
    }
}
