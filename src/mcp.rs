use std::collections::HashMap;

use crate::sp::Task;

pub fn render_tasks(tasks: &[Task], projects: &HashMap<String, String>) -> String {
    tasks
        .iter()
        .map(|t| {
            let project_label = match projects.get(&t.project_id) {
                Some(id) => id,
                None => &t.project_id,
            };
            match t.time_estimate {
                Some(value) => format!(
                    "{} - {} min - project: {} - id: {}",
                    t.title,
                    value / 60_000,
                    project_label,
                    t.id
                ),
                None => format!("{} - project: {} - id: {}", t.title, project_label, t.id),
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::mcp::render_tasks;
    use crate::sp::Task;
    use std::collections::HashMap;

    // Only proj_abc has a name here, so an id outside the map exercises the fallback.
    fn project_names() -> HashMap<String, String> {
        HashMap::from([("proj_abc".to_owned(), "tsfm".to_owned())])
    }

    #[test]
    fn render_tasks_reports_estimates_in_minutes() {
        let tasks = vec![
            Task {
                id: "task_aaa".to_owned(),
                title: "check dynatrace access".to_owned(),
                time_estimate: Some(900_000),
                project_id: "proj_abc".to_owned(),
            },
            Task {
                id: "task_bbb".to_owned(),
                title: "triage inbox".to_owned(),
                time_estimate: None,
                project_id: "proj_abc".to_owned(),
            },
        ];

        let text = render_tasks(&tasks, &project_names());

        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);

        let with_estimate = lines
            .iter()
            .find(|line| line.contains("check dynatrace access"))
            .expect("the estimated task should have its own line");
        assert!(with_estimate.contains("15 min"));
        assert!(with_estimate.contains("project: tsfm"));
        assert!(with_estimate.contains("id: task_aaa"));
        assert!(!with_estimate.contains("900000"));

        let without_estimate = lines
            .iter()
            .find(|line| line.contains("triage inbox"))
            .expect("the unestimated task should have its own line");
        assert!(without_estimate.contains("project: tsfm"));
        assert!(without_estimate.contains("id: task_bbb"));
        assert!(!without_estimate.contains("min"));
    }

    #[test]
    fn an_unknown_project_id_falls_back_to_the_id() {
        let tasks = vec![Task {
            id: "task_zzz".to_owned(),
            title: "orphan task".to_owned(),
            time_estimate: Some(600_000),
            project_id: "proj_zzz".to_owned(),
        }];

        let text = render_tasks(&tasks, &project_names());

        assert!(text.contains("orphan task"));
        assert!(text.contains("proj_zzz"));
        assert!(text.contains("id: task_zzz"));
    }
}
