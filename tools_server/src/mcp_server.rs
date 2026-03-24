use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::{RequestContext, RoleServer},
};
use serde_json::{Map, Value};
use tokio::io::{stdin, stdout};

use crate::tools;

pub async fn run_mcp_stdio() -> anyhow::Result<()> {
    tracing::info!("starting MCP stdio server");

    let server = ToolsMcpServer;

    let running = server.serve((stdin(), stdout())).await?;
    let _quit_reason = running.waiting().await?;

    Ok(())
}

#[derive(Clone)]
struct ToolsMcpServer;

impl ServerHandler for ToolsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let defs = tools::all_definitions();

        let mut mcp_tools = Vec::with_capacity(defs.len());

        for def in defs {
            let schema_obj: Map<String, Value> = match def.input_schema {
                Value::Object(map) => map,
                other => {
                    return Err(McpError::internal_error(
                        "tool input_schema must be a JSON object",
                        Some(other),
                    ));
                }
            };

            let tool =
                Tool::new_with_raw(def.name, Some(def.description.into()), Arc::new(schema_obj));

            mcp_tools.push(tool);
        }

        Ok(ListToolsResult::with_all_items(mcp_tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let arguments_map = request.arguments.unwrap_or_default();
        let arguments = Value::Object(arguments_map);

        let result = tools::execute(shared_types::ToolCallRequest {
            name: request.name.to_string(),
            arguments,
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::structured(result.content))
    }
}
