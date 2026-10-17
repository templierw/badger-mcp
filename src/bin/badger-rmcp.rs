use std::{collections::HashMap, vec};

use badger_mcp::{mcp::render_tasks, sp};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router,
    transport::stdio,
};

#[derive(Debug, Clone)]
pub struct Badger {
    tool_router: ToolRouter<Self>,
    client: sp::SpClient,
}

impl Badger {
    pub fn new(token: String) -> Self {
        Self {
            tool_router: Self::tool_router(),
            client: sp::SpClient::new(token),
        }
    }
}

#[tool_router]
impl Badger {
    #[tool(
        description = "Return task titles, time estimates (in min) and project names. Estimate can be missing."
    )]
    async fn list_tasks(&self) -> Result<CallToolResult, McpError> {
        let tasks = self.client.fetch_tasks().await;
        let projects: HashMap<String, String> = match self.client.fetch_projects().await {
            Ok(ps) => ps.into_iter().map(|p| (p.id, p.title)).collect(),
            Err(e) => {
                eprintln!("{}", e);
                HashMap::new()
            }
        };

        match tasks {
            Ok(t) => Ok(CallToolResult::success(vec![ContentBlock::text(
                render_tasks(&t, &projects),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(e)])),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Badger {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("badger-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions("Read tasks from the local Super Productivity app.")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("SPROD_TOKEN").expect("Missing SPROD_TOKEN. Aborting.");
    let service = Badger::new(token).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
