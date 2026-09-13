use serde::Deserialize;

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

fn main() {
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .build()
        .expect("should build client");

    let response = client
        .get("http://127.0.0.1:3876/health")
        .send()
        .expect("bad response");

    match response.status() {
        reqwest::StatusCode::OK => {
            println!("Server is healthy");
            println!("{:}", response.text().expect("should read response text"));
        }
        err => println!("{:}", err.as_u16()),
    }

    let response = client
        .get("http://127.0.0.1:3876/tasks")
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                std::env::var("SPROD_TOKEN").expect("SPROD_TOKEN not set")
            ),
        )
        .send()
        .expect("bad response");
    match response.status() {
        reqwest::StatusCode::OK => {
            println!("Tasks fetched successfully");

            let body = serde_json::from_str::<Envelope<Vec<Task>>>(
                &response.text().expect("should read response text"),
            )
            .expect("should deserialize");

            println!("{:#?}", body);
        }
        err => println!("{:}", err.as_u16()),
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct SpError {
    pub code: String,
    pub message: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum Envelope<T> {
    Success { data: T },
    Failure { error: SpError },
}

fn zero_as_none<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;
    if value == 0 {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub title: String,
    #[serde(default, deserialize_with = "zero_as_none")]
    pub time_estimate: Option<u64>,
    pub project_id: String,
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
        _ => Response::error(id, -32601, "Method not found".to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use crate::{Envelope, Id, Request, Response, RpcError, Task, handle};

    #[test]
    fn success_envelope_yields_success_variant() {
        let json = r#"{"ok":true,"data":42}"#;

        let env: Envelope<i32> = serde_json::from_str(json).expect("should deserialize");

        match env {
            Envelope::Success { data } => assert_eq!(data, 42),
            Envelope::Failure { .. } => panic!("success envelope parsed as the failure variant"),
        }
    }

    #[test]
    fn failure_envelope_yields_failure_variant() {
        let json = r#"{"ok":false,"error":{"code":"UNAUTHORIZED","message":"Bad token"}}"#;

        let env: Envelope<i32> = serde_json::from_str(json).expect("should deserialize");

        match env {
            Envelope::Failure { error } => assert_eq!(error.code, "UNAUTHORIZED"),
            Envelope::Success { .. } => panic!("failure envelope parsed as the success variant"),
        }
    }

    #[test]
    fn task_reads_the_three_exposed_fields() {
        let json = r#"{
            "id": "abc123",
            "title": "TSFM - Cross Stream Review",
            "timeEstimate": 3600000,
            "projectId": "proj_xyz",
            "notes": "some notes",
            "isDone": false
        }"#;

        let task: Task = serde_json::from_str(json).expect("should deserialize");

        assert_eq!(task.title, "TSFM - Cross Stream Review");
        assert_eq!(task.time_estimate, Some(3600000));
        assert_eq!(task.project_id, "proj_xyz");
    }

    #[test]
    fn a_task_without_a_time_estimate_still_parses() {
        let json = r#"{
            "ok": true,
            "data": [
                {"title":"bdd-pr","timeEstimate":1800000,"projectId":"proj_abc"},
                {"title":"triage inbox","projectId":"proj_abc"}
            ]
        }"#;

        let env: Envelope<Vec<Task>> = serde_json::from_str(json).expect("should deserialize");

        match env {
            Envelope::Success { data } => {
                assert_eq!(data.len(), 2);
                assert_eq!(data[1].title, "triage inbox");
                assert_eq!(data[1].time_estimate, None);
            }
            Envelope::Failure { .. } => panic!("success envelope parsed as the failure variant"),
        }
    }

    #[test]
    fn a_zero_time_estimate_means_no_estimate() {
        let json = r#"{"title":"badger","timeEstimate":0,"projectId":"proj_abc"}"#;

        let task: Task = serde_json::from_str(json).expect("should deserialize");

        assert_eq!(task.time_estimate, None);
    }

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
}
