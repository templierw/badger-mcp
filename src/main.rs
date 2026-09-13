use badger_mcp::sp::{Envelope, Task};


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

