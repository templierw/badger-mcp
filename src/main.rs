use std::{eprintln, io::stdin};

use badger_mcp::{mcp, sp};

fn main() {
    let token = std::env::var("SPROD_TOKEN").expect("Missing SPROD_TOKEN. Aborting.");
    let client = sp::SpClient::new(token);

    for line in stdin().lines() {
        match line {
            Ok(content) => match serde_json::from_str::<mcp::Request>(&content) {
                Ok(request) => {
                    if let Some(response) = mcp::handle(request, &client) {
                        println!("{}", serde_json::to_string(&response).unwrap())
                    }
                }
                Err(e) => eprintln!("malformed request {}", e),
            },
            Err(e) => eprintln!("failed to read line {}", e),
        }
    }
}
