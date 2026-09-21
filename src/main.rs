use std::{eprintln, io::stdin};

use badger_mcp::mcp;

fn main() {
    let lines = stdin().lines();

    for line in lines {
        match line {
            Ok(content) => match serde_json::from_str::<mcp::Request>(&content) {
                Ok(request) => {
                    if let Some(response) = mcp::handle(request) {
                        println!("{}", serde_json::to_string(&response).unwrap())
                    }
                }
                Err(e) => eprintln!("malformed request {}", e),
            },
            Err(e) => eprintln!("failed to read line {}", e),
        }
    }
}
