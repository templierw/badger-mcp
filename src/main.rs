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

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub title: String,
    pub time_estimate: Option<u64>,
    pub project_id: String,
}

#[cfg(test)]
mod tests {
    use crate::{Envelope, Task};

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
}
