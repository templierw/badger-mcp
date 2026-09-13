use serde::Deserialize;

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


#[cfg(test)]
mod tests {
    use crate::sp::{Envelope, Task};

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
}