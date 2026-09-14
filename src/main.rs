use std::{eprintln, io::stdin};

use badger_mcp::mcp;


fn main() {
    let lines = stdin().lines();

    for line in lines {
        match line {
            Ok(content) => {
                match serde_json::from_str::<mcp::Request>(&content) {
                    Ok(request) => {
                        match mcp::handle(request) {
                            Some(response) => println!("{}", serde_json::to_string(&response).unwrap()),
                            None => (),
                        }
                    },
                    Err(e) => eprintln!("malformed request {}", e),
                }
            },
            Err(e) => eprintln!("failed to read line {}", e),
        }
    }
}

